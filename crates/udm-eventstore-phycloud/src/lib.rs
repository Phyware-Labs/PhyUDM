//! PhyCloud adapter for [`UdmEventStore`].
//!
//! Implements the [`UdmEventStore`] contract against a live PhyCloud
//! deployment over its REST API (`GET /api/v1/events` and
//! `GET /api/v1/events/{id}`). PhyCloud is the read/write telemetry store;
//! this adapter is the **read** side that the `udm` CLI and the `udm-mcp`
//! server query through.
//!
//! # Design
//!
//! PhyCloud indexes a fixed set of columns (`source_id`, `source_type`,
//! `event_type`, `session_id`, and the capture-time range). The adapter
//! **pushes those predicates down** to the server as query-string filters and
//! evaluates any remaining [`Predicate`] terms client-side via
//! [`Predicate::matches`], fetching the full UDM payload with
//! `include=event_data`. Aggregation and timeline ordering are likewise
//! computed client-side over the fetched page set.
//!
//! # Session scoping
//!
//! A single PhyCloud deployment can hold many missions side by side. Set
//! [`PhyCloudConfig::session_id`] (e.g. from a `phycloud://…?session_id=<mission>`
//! URL) to constrain every query to one mission's events.

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

use std::time::Duration;

use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use serde::Deserialize;
use serde_json::Value;

use udm_eventstore::{
    AggregateQuery, AggregateResult, Error, EventId, EventPage, EventQuery, EventStream,
    GetEventOptions, OrderBy, Result, SourceId, StoreCapabilities, TimeRange, UdmEvent,
    UdmEventStore,
};

mod aggregate;
mod pushdown;

#[cfg(test)]
mod tests;

use aggregate::compute_aggregate;
use pushdown::{extract_pushdown, PushFilter};

/// Largest page size the adapter requests from PhyCloud.
const PAGE_SIZE: usize = 1000;

/// Connection configuration for the PhyCloud adapter.
#[derive(Debug, Clone)]
pub struct PhyCloudConfig {
    /// Base URL of the PhyCloud HTTP API (e.g.
    /// `https://api.phycloud.example.com` or `http://localhost:8000`).
    /// The `/api/v1` path is appended by the adapter.
    pub endpoint: String,
    /// API key or bearer token. Sent as `Authorization: Bearer <api_key>`.
    pub api_key: String,
    /// Optional tenant override for multi-tenant deployments.
    pub tenant_id: Option<String>,
    /// Optional mission/session scope. When set, every query is
    /// constrained to `session_id = <this>` so many missions can share
    /// one deployment without cross-talk.
    pub session_id: Option<String>,
    /// Request timeout in seconds (default 30).
    pub timeout_secs: u64,
}

impl PhyCloudConfig {
    /// Build a config from explicit endpoint + api_key. Tenant / session
    /// default to unset; timeout defaults to 30 s.
    pub fn new(endpoint: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            api_key: api_key.into(),
            tenant_id: None,
            session_id: None,
            timeout_secs: 30,
        }
    }

    /// Scope all queries to a single mission/session id.
    #[must_use]
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// The `/api/v1/events` base URL derived from `endpoint`.
    fn events_url(&self) -> String {
        format!("{}/api/v1/events", self.endpoint.trim_end_matches('/'))
    }
}

/// PhyCloud-backed implementation of [`UdmEventStore`].
pub struct PhyCloudStore {
    config: PhyCloudConfig,
    http: reqwest::Client,
    capabilities: StoreCapabilities,
}

impl PhyCloudStore {
    /// Construct a PhyCloud store from the given config.
    ///
    /// Builds the HTTP client eagerly but does no network I/O; the first
    /// request opens the connection.
    pub fn new(config: PhyCloudConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs.max(1)))
            .build()
            .unwrap_or_default();
        let capabilities = StoreCapabilities {
            backend: "phycloud".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            // Aggregation, full-text (Contains), and ordered streaming are
            // all served by the adapter (client-side over fetched pages).
            supports_aggregation: true,
            supports_full_text: true,
            supports_ordered_streaming: true,
            max_page_size: PAGE_SIZE,
            indexed_paths: vec![
                "event_id".to_owned(),
                "event_type".to_owned(),
                "source_id".to_owned(),
                "source_type".to_owned(),
                "session_id".to_owned(),
                "captured_at".to_owned(),
            ],
            cursor_scheme: "offset".to_owned(),
        };
        Self {
            config,
            http,
            capabilities,
        }
    }

    /// Borrow the configured connection settings for diagnostics.
    pub fn config(&self) -> &PhyCloudConfig {
        &self.config
    }

    /// Fetch every event matching `push` by paging through PhyCloud until
    /// the server reports no more pages.
    async fn fetch_all(&self, push: &PushFilter) -> Result<Vec<UdmEvent>> {
        let mut out = Vec::new();
        let mut page: u32 = 1;
        loop {
            let (mut events, has_more) = self.fetch_page(push, page).await?;
            out.append(&mut events);
            if !has_more {
                break;
            }
            page += 1;
        }
        Ok(out)
    }

    /// Fetch one page of full events from `GET /api/v1/events`.
    async fn fetch_page(&self, push: &PushFilter, page: u32) -> Result<(Vec<UdmEvent>, bool)> {
        let mut params: Vec<(String, String)> = vec![
            ("include".to_owned(), "event_data".to_owned()),
            ("page".to_owned(), page.to_string()),
            ("page_size".to_owned(), PAGE_SIZE.to_string()),
        ];
        push.apply(&mut params, self.config.session_id.as_deref());

        let resp = self
            .http
            .get(self.config.events_url())
            .bearer_auth(&self.config.api_key)
            .query(&params)
            .send()
            .await
            .map_err(map_reqwest)?;

        let resp = check_status(resp)?;
        let body: ListResponse = resp.json().await.map_err(map_reqwest)?;

        let mut events = Vec::with_capacity(body.events.len());
        for item in &body.events {
            events.push(parse_event_data(item)?);
        }
        Ok((events, body.has_more))
    }
}

/// Minimal view of PhyCloud's `EventListResponse` (we only need the
/// payloads and the pagination flag).
#[derive(Debug, Deserialize)]
struct ListResponse {
    events: Vec<Value>,
    #[serde(default)]
    has_more: bool,
}

/// Pull the canonical UDM event out of an `event_data`-bearing list item
/// (or a `GET /events/{id}` response body).
fn parse_event_data(item: &Value) -> Result<UdmEvent> {
    let payload = item
        .get("event_data")
        .ok_or_else(|| Error::Backend("PhyCloud response item has no event_data".to_owned()))?;
    serde_json::from_value(payload.clone()).map_err(Error::from)
}

fn map_reqwest(err: reqwest::Error) -> Error {
    if err.is_timeout() || err.is_connect() {
        Error::Unavailable(err.to_string())
    } else {
        Error::Backend(err.to_string())
    }
}

fn check_status(resp: reqwest::Response) -> Result<reqwest::Response> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    let code = status.as_u16();
    let msg = format!("PhyCloud returned HTTP {code}");
    match code {
        401 | 403 => Err(Error::Forbidden(msg)),
        404 => Err(Error::NotFound(msg)),
        502..=504 => Err(Error::Unavailable(msg)),
        _ => Err(Error::Backend(msg)),
    }
}

/// Sort events per `order_by` (in place).
fn sort_events(events: &mut [UdmEvent], order_by: OrderBy) {
    match order_by {
        OrderBy::CapturedAtAsc | OrderBy::ReceivedAtAsc => {
            events.sort_by_key(|e| e.captured_at);
        }
        OrderBy::CapturedAtDesc => {
            events.sort_by_key(|e| std::cmp::Reverse(e.captured_at));
        }
    }
}

#[async_trait]
impl UdmEventStore for PhyCloudStore {
    async fn get_event(&self, id: &EventId, _opts: &GetEventOptions) -> Result<Option<UdmEvent>> {
        let url = format!("{}/{}", self.config.events_url(), id);
        let resp = self
            .http
            .get(url)
            .bearer_auth(&self.config.api_key)
            .send()
            .await
            .map_err(map_reqwest)?;

        if resp.status().as_u16() == 404 {
            return Ok(None);
        }
        let resp = check_status(resp)?;
        let body: Value = resp.json().await.map_err(map_reqwest)?;
        let event = parse_event_data(&body)?;

        // Enforce session scoping: an id from another mission is "not here".
        if let Some(scope) = &self.config.session_id {
            if event.session_id.as_deref() != Some(scope.as_str()) {
                return Ok(None);
            }
        }
        Ok(Some(event))
    }

    async fn query_events(&self, q: &EventQuery) -> Result<EventPage> {
        let mut push = PushFilter::default();
        if let Some(sid) = &q.source_id {
            push.source_id = Some(sid.clone());
        }
        if let Some(tr) = q.time_range {
            push.start = Some(tr.start);
            push.end = Some(tr.end);
        }
        extract_pushdown(&q.predicate, &mut push);

        let mut events = self.fetch_all(&push).await?;
        // Apply the full predicate + explicit source filter client-side so
        // terms PhyCloud can't index are still honoured.
        events.retain(|e| q.predicate.matches(e));
        if let Some(sid) = &q.source_id {
            events.retain(|e| &e.source_id == sid);
        }
        sort_events(&mut events, q.order_by);

        let offset: usize = q
            .cursor
            .as_deref()
            .and_then(|c| c.parse().ok())
            .unwrap_or(0);
        let limit = q.limit.min(self.capabilities.max_page_size).max(1);
        let total = events.len() as u64;

        let window: Vec<UdmEvent> = events.into_iter().skip(offset).take(limit).collect();
        let next = offset + window.len();
        let next_cursor = (next < total as usize).then(|| next.to_string());

        Ok(EventPage {
            events: window,
            next_cursor,
            total_estimate: Some(total),
        })
    }

    async fn timeline(&self, source_id: &SourceId, range: TimeRange) -> Result<EventStream<'_>> {
        let push = PushFilter {
            source_id: Some(source_id.clone()),
            start: Some(range.start),
            end: Some(range.end),
            ..PushFilter::default()
        };
        let mut events = self.fetch_all(&push).await?;
        events.retain(|e| &e.source_id == source_id && range.contains(e.captured_at));
        sort_events(&mut events, OrderBy::CapturedAtAsc);
        Ok(stream::iter(events.into_iter().map(Ok)).boxed())
    }

    async fn aggregate(&self, agg: &AggregateQuery) -> Result<AggregateResult> {
        let mut push = PushFilter::default();
        if let Some(tr) = agg.time_range {
            push.start = Some(tr.start);
            push.end = Some(tr.end);
        }
        extract_pushdown(&agg.predicate, &mut push);

        let mut events = self.fetch_all(&push).await?;
        events.retain(|e| agg.predicate.matches(e));
        if let Some(tr) = agg.time_range {
            events.retain(|e| tr.contains(e.captured_at));
        }
        compute_aggregate(agg, &events)
    }

    fn capabilities(&self) -> &StoreCapabilities {
        &self.capabilities
    }
}
