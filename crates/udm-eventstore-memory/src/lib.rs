//! In-process [`UdmEventStore`] adapter backed by a `Vec<UdmEvent>`.
//!
//! **Scope:** testing, demos, and cookbook reproducibility. Not a
//! production deployment target — see
//! [`udm-eventstore-phycloud`](https://crates.io/crates/udm-eventstore-phycloud)
//! for the reference deployment adapter.
//!
//! The store is constructed either from an in-memory
//! `Vec<UdmEvent>` (handy for unit tests) or by streaming an NDJSON
//! file (handy for cookbook articles — `memory:///path/to/fixture.ndjson`).
//!
//! # Quick start
//!
//! ```no_run
//! # async fn run() -> udm_eventstore::Result<()> {
//! use udm_eventstore_memory::MemoryStore;
//!
//! // Load events from disk:
//! let store = MemoryStore::from_ndjson_path("docs/ai-cookbook/datasets/fleet-q1.ndjson").await?;
//!
//! // Or construct directly:
//! use udm_eventstore::conformance::load_fixture_events;
//! let store = MemoryStore::from_events(load_fixture_events());
//! # Ok(()) }
//! ```

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use tokio::io::AsyncBufReadExt;

use udm_eventstore::{
    AggregateBucket, AggregateFn, AggregateQuery, AggregateResult, Error, EventId, EventPage,
    EventQuery, EventStream, GetEventOptions, OrderBy, Result, SourceId, StoreCapabilities,
    TimeRange, UdmEvent, UdmEventStore,
};

/// In-process implementation of [`UdmEventStore`].
#[derive(Clone)]
pub struct MemoryStore {
    inner: Arc<MemoryStoreInner>,
}

struct MemoryStoreInner {
    /// Events kept in `captured_at` ascending order so timeline queries
    /// don't have to re-sort.
    events: Vec<UdmEvent>,
    capabilities: StoreCapabilities,
}

impl MemoryStore {
    /// Construct from an explicit `Vec<UdmEvent>`. Events are sorted
    /// by `captured_at` ascending on construction.
    pub fn from_events(mut events: Vec<UdmEvent>) -> Self {
        events.sort_by_key(|e| e.captured_at);
        Self {
            inner: Arc::new(MemoryStoreInner {
                events,
                capabilities: default_capabilities(),
            }),
        }
    }

    /// Load events from an NDJSON file on disk.
    pub async fn from_ndjson_path(path: impl AsRef<Path>) -> Result<Self> {
        let file = tokio::fs::File::open(path.as_ref()).await?;
        let reader = tokio::io::BufReader::new(file);
        let mut events = Vec::new();
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }
            let event: UdmEvent = serde_json::from_str(&line)?;
            events.push(event);
        }
        Ok(Self::from_events(events))
    }

    /// Construct from an NDJSON string (in-memory; useful for tests).
    pub fn from_ndjson_str(ndjson: &str) -> Result<Self> {
        let events: Vec<UdmEvent> = ndjson
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(serde_json::from_str)
            .collect::<std::result::Result<_, _>>()?;
        Ok(Self::from_events(events))
    }

    /// Number of events held by the store.
    pub fn len(&self) -> usize {
        self.inner.events.len()
    }

    /// `true` when the store holds no events.
    pub fn is_empty(&self) -> bool {
        self.inner.events.is_empty()
    }
}

fn default_capabilities() -> StoreCapabilities {
    StoreCapabilities {
        backend: "memory".to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
        supports_aggregation: true,
        supports_full_text: true,
        supports_ordered_streaming: true,
        max_page_size: 10_000,
        indexed_paths: vec![
            "event_id".to_owned(),
            "event_type".to_owned(),
            "source_id".to_owned(),
            "source_type".to_owned(),
            "captured_at".to_owned(),
        ],
        cursor_scheme: "offset".to_owned(),
    }
}

#[async_trait]
impl UdmEventStore for MemoryStore {
    async fn get_event(&self, id: &EventId, _opts: &GetEventOptions) -> Result<Option<UdmEvent>> {
        Ok(self
            .inner
            .events
            .iter()
            .find(|e| &e.event_id == id)
            .cloned())
    }

    async fn query_events(&self, q: &EventQuery) -> Result<EventPage> {
        if q.limit == 0 {
            return Err(Error::InvalidQuery("limit must be > 0".into()));
        }
        let limit = q.limit.min(self.inner.capabilities.max_page_size);

        // Filter.
        let mut filtered: Vec<UdmEvent> = self
            .inner
            .events
            .iter()
            .filter(|e| match q.time_range {
                Some(range) => range.contains(e.captured_at),
                None => true,
            })
            .filter(|e| match &q.source_id {
                Some(sid) => &e.source_id == sid,
                None => true,
            })
            .filter(|e| q.predicate.matches(e))
            .cloned()
            .collect();

        // Sort.
        match q.order_by {
            OrderBy::CapturedAtAsc => filtered.sort_by_key(|e| e.captured_at),
            OrderBy::CapturedAtDesc => {
                filtered.sort_by_key(|e| std::cmp::Reverse(e.captured_at));
            }
            OrderBy::ReceivedAtAsc => {
                filtered.sort_by_key(|e| e.received_at.unwrap_or(e.captured_at));
            }
        }

        // Apply cursor (offset-based).
        let offset = match q.cursor.as_deref() {
            None => 0,
            Some(s) => s
                .parse::<usize>()
                .map_err(|err| Error::InvalidQuery(format!("invalid cursor {s:?}: {err}")))?,
        };

        let total = filtered.len() as u64;
        let end = (offset + limit).min(filtered.len());
        let next_cursor = if end < filtered.len() {
            Some(end.to_string())
        } else {
            None
        };
        let events = filtered.into_iter().skip(offset).take(limit).collect();

        Ok(EventPage {
            events,
            next_cursor,
            total_estimate: Some(total),
        })
    }

    async fn timeline(&self, source_id: &SourceId, range: TimeRange) -> Result<EventStream<'_>> {
        let source_id = source_id.clone();
        // Already sorted ascending by construction of the store; just
        // project the matching subset into the stream.
        let iter = self
            .inner
            .events
            .iter()
            .filter(move |e| e.source_id == source_id && range.contains(e.captured_at))
            .cloned()
            .map(Ok);
        Ok(stream::iter(iter).boxed())
    }

    async fn aggregate(&self, agg: &AggregateQuery) -> Result<AggregateResult> {
        if agg.agg_fn != AggregateFn::Count && agg.field.is_none() {
            return Err(Error::InvalidQuery(
                "aggregate field is required for non-count aggregations".into(),
            ));
        }

        let mut buckets: BTreeMap<Vec<String>, BucketAccum> = BTreeMap::new();
        for event in &self.inner.events {
            if let Some(range) = agg.time_range {
                if !range.contains(event.captured_at) {
                    continue;
                }
            }
            if !agg.predicate.matches(event) {
                continue;
            }

            let key = group_key(event, &agg.group_by)?;
            let entry = buckets.entry(key.clone()).or_default();
            entry.count += 1;
            entry.key_values = key;
            entry.key_paths.clone_from(&agg.group_by);

            if agg.agg_fn != AggregateFn::Count {
                let field = agg.field.as_deref().expect("checked above");
                if let Some(value) = pointer_f64(event, field) {
                    entry.values.push(value);
                }
            }
        }

        let mut out = Vec::with_capacity(buckets.len());
        for (_, accum) in buckets {
            let value = match agg.agg_fn {
                AggregateFn::Count => accum.count as f64,
                AggregateFn::Sum => accum.values.iter().sum(),
                AggregateFn::Avg => {
                    if accum.values.is_empty() {
                        continue;
                    }
                    accum.values.iter().sum::<f64>() / accum.values.len() as f64
                }
                AggregateFn::Min => match accum.values.iter().copied().reduce(f64::min) {
                    Some(v) => v,
                    None => continue,
                },
                AggregateFn::Max => match accum.values.iter().copied().reduce(f64::max) {
                    Some(v) => v,
                    None => continue,
                },
            };
            let mut key_map = BTreeMap::new();
            for (path, value) in accum.key_paths.iter().zip(accum.key_values.iter()) {
                key_map.insert(path.clone(), serde_json::Value::String(value.clone()));
            }
            out.push(AggregateBucket {
                key: key_map,
                value,
                event_count: accum.count,
            });
        }

        Ok(AggregateResult {
            buckets: out,
            agg_fn: agg.agg_fn,
        })
    }

    fn capabilities(&self) -> &StoreCapabilities {
        &self.inner.capabilities
    }
}

#[derive(Default, Clone)]
struct BucketAccum {
    count: u64,
    values: Vec<f64>,
    key_values: Vec<String>,
    key_paths: Vec<String>,
}

fn group_key(event: &UdmEvent, paths: &[String]) -> Result<Vec<String>> {
    if paths.is_empty() {
        return Ok(vec![String::new()]);
    }
    let value = serde_json::to_value(event)?;
    Ok(paths
        .iter()
        .map(|path| {
            let trimmed = path.trim_start_matches('/');
            let ptr = format!("/{trimmed}");
            value.pointer(&ptr).map_or_else(
                || "<null>".into(),
                |v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                },
            )
        })
        .collect())
}

fn pointer_f64(event: &UdmEvent, path: &str) -> Option<f64> {
    let value = serde_json::to_value(event).ok()?;
    let trimmed = path.trim_start_matches('/');
    let ptr = format!("/{trimmed}");
    value.pointer(&ptr).and_then(serde_json::Value::as_f64)
}

// (Removed unused captured_at helper.)

#[cfg(test)]
mod tests {
    use super::*;
    use udm_eventstore::{conformance::load_fixture_events, Predicate};

    fn store() -> MemoryStore {
        MemoryStore::from_events(load_fixture_events())
    }

    #[tokio::test]
    async fn from_ndjson_str_roundtrips() {
        let store =
            MemoryStore::from_ndjson_str(udm_eventstore::conformance::FIXTURE_NDJSON).unwrap();
        assert_eq!(store.len(), 6);
    }

    #[tokio::test]
    async fn aggregate_avg_field() {
        let store = store();
        let agg = AggregateQuery {
            agg_fn: AggregateFn::Avg,
            field: Some("power/battery/soc_pct".into()),
            group_by: vec!["source_id".into()],
            time_range: None,
            predicate: Predicate::always(),
        };
        let result = store.aggregate(&agg).await.unwrap();
        let amr = result
            .buckets
            .iter()
            .find(|b| b.key.get("source_id") == Some(&serde_json::json!("amr-001")))
            .expect("amr bucket");
        // amr-001 has two power events: 78.4, 78.2 → 78.3
        let avg = amr.value;
        assert!((avg - 78.3).abs() < 1e-9, "expected ~78.3, got {avg}");
    }
}
