//! Reusable conformance suite for `UdmEventStore` adapters.
//!
//! Any adapter — community or in-tree — SHOULD pass the suite in
//! [`run_full_suite`] to be considered behaviourally compatible with
//! UDM analysis tooling. The suite is deliberately small (single
//! deterministic fixture loaded from
//! [`FIXTURE_NDJSON`]) so adapters that wrap remote services can
//! exercise it against a seeded test instance.
//!
//! # Usage from an adapter crate
//!
//! ```ignore
//! use udm_eventstore::conformance::{load_fixture_events, run_full_suite};
//!
//! #[tokio::test]
//! async fn passes_conformance() {
//!     let events = load_fixture_events();
//!     let store = MyAdapter::with_events(events);
//!     run_full_suite(&store).await.expect("adapter must pass the suite");
//! }
//! ```

use chrono::{DateTime, Utc};
use futures::StreamExt;

use crate::event::UdmEvent;
use crate::query::GetEventOptions;
use crate::query::{AggregateFn, AggregateQuery, EventQuery, Predicate, TimeRange};
use crate::store::UdmEventStore;
use crate::Result;

/// Deterministic fixture used by the conformance suite. 6 events
/// spanning 3 sources, 4 event types, and 4 domains.
///
/// Adapters MAY hydrate themselves from this NDJSON via
/// [`load_fixture_events`].
pub const FIXTURE_NDJSON: &str = include_str!("../test-data/conformance.ndjson");

/// Parse [`FIXTURE_NDJSON`] into a `Vec<UdmEvent>` for an adapter to
/// hydrate itself with.
pub fn load_fixture_events() -> Vec<UdmEvent> {
    FIXTURE_NDJSON
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str::<UdmEvent>(line)
                .unwrap_or_else(|err| panic!("invalid fixture event: {err}"))
        })
        .collect()
}

/// Run every conformance check against `store`.
///
/// Adapters SHOULD call this from a `#[tokio::test]` after hydrating
/// the store with [`load_fixture_events`]. Returns the first error
/// encountered; passing this suite is a precondition for being listed
/// in the OSS adapter registry.
pub async fn run_full_suite<S: UdmEventStore>(store: &S) -> Result<()> {
    check_capabilities(store);
    check_get_event(store).await?;
    check_query_events_paging(store).await?;
    check_query_events_predicate(store).await?;
    check_query_events_time_range(store).await?;
    check_timeline_ordering(store).await?;
    if store.capabilities().supports_aggregation {
        check_aggregate(store).await?;
    }
    Ok(())
}

fn check_capabilities<S: UdmEventStore>(store: &S) {
    let caps = store.capabilities();
    assert!(!caps.backend.is_empty(), "backend name must be non-empty");
    assert!(
        !caps.version.is_empty(),
        "backend version must be non-empty"
    );
    assert!(caps.max_page_size > 0, "max_page_size must be positive");
}

async fn check_get_event<S: UdmEventStore>(store: &S) -> Result<()> {
    let event = store
        .get_event(
            &"01940000-0000-7000-8000-000000000001".to_string(),
            &GetEventOptions::default(),
        )
        .await?
        .expect("conformance fixture id 1 must exist");
    assert_eq!(event.source_id, "amr-001");
    assert_eq!(event.event_type, "telemetry_periodic");

    let missing = store
        .get_event(&"does-not-exist".to_string(), &GetEventOptions::default())
        .await?;
    assert!(missing.is_none(), "unknown id must return Ok(None)");
    Ok(())
}

async fn check_query_events_paging<S: UdmEventStore>(store: &S) -> Result<()> {
    let q = EventQuery {
        limit: 2,
        ..EventQuery::default()
    };
    let page1 = store.query_events(&q).await?;
    assert!(page1.events.len() <= 2);
    if page1.events.len() == 2 {
        assert!(
            page1.next_cursor.is_some(),
            "page that hits the limit must return a cursor"
        );
    }

    let q_all = EventQuery {
        limit: 100,
        ..EventQuery::default()
    };
    let all = store.query_events(&q_all).await?;
    assert!(
        all.events.len() >= 6,
        "fixture has 6 events; got {}",
        all.events.len()
    );
    Ok(())
}

async fn check_query_events_predicate<S: UdmEventStore>(store: &S) -> Result<()> {
    let q = EventQuery {
        predicate: Predicate::Eq {
            field: "event_type".into(),
            value: serde_json::Value::String("safety_violation".into()),
        },
        limit: 100,
        ..EventQuery::default()
    };
    let page = store.query_events(&q).await?;
    assert!(
        !page.events.is_empty(),
        "fixture has at least one safety_violation"
    );
    assert!(page
        .events
        .iter()
        .all(|e| e.event_type == "safety_violation"));
    Ok(())
}

async fn check_query_events_time_range<S: UdmEventStore>(store: &S) -> Result<()> {
    let start: DateTime<Utc> = "2026-06-07T19:00:00Z".parse().unwrap();
    let end: DateTime<Utc> = "2026-06-07T19:00:10Z".parse().unwrap();
    let q = EventQuery {
        time_range: Some(TimeRange::new(start, end)?),
        limit: 100,
        ..EventQuery::default()
    };
    let page = store.query_events(&q).await?;
    for event in &page.events {
        assert!(
            event.captured_at >= start && event.captured_at < end,
            "event {} falls outside [{start},{end})",
            event.event_id
        );
    }
    Ok(())
}

async fn check_timeline_ordering<S: UdmEventStore>(store: &S) -> Result<()> {
    let start: DateTime<Utc> = "2026-06-07T00:00:00Z".parse().unwrap();
    let end: DateTime<Utc> = "2026-06-08T00:00:00Z".parse().unwrap();
    let mut stream = store
        .timeline(&"amr-001".to_string(), TimeRange::new(start, end)?)
        .await?;
    let mut last_ts: Option<DateTime<Utc>> = None;
    let mut count = 0usize;
    while let Some(item) = stream.next().await {
        let event = item?;
        assert_eq!(
            event.source_id, "amr-001",
            "timeline must only emit events for the requested source_id"
        );
        if let Some(prev) = last_ts {
            assert!(
                event.captured_at >= prev,
                "timeline events must be ascending by captured_at"
            );
        }
        last_ts = Some(event.captured_at);
        count += 1;
    }
    assert!(count >= 3, "fixture has ≥3 events for amr-001; got {count}");
    Ok(())
}

async fn check_aggregate<S: UdmEventStore>(store: &S) -> Result<()> {
    let agg = AggregateQuery {
        agg_fn: AggregateFn::Count,
        field: None,
        group_by: vec!["source_id".into()],
        time_range: None,
        predicate: Predicate::always(),
    };
    let result = store.aggregate(&agg).await?;
    assert_eq!(result.agg_fn, AggregateFn::Count);
    assert!(
        !result.buckets.is_empty(),
        "count grouped by source_id should produce at least one bucket"
    );
    let total: u64 = result.buckets.iter().map(|b| b.event_count).sum();
    assert_eq!(total, 6, "fixture has 6 events; saw {total}");
    Ok(())
}
