//! Unit tests for the pure (non-network) parts of the adapter: config,
//! predicate push-down, sorting, payload parsing, and aggregation.

use serde_json::json;
use udm_eventstore::{
    AggregateFn, AggregateQuery, EventId, GetEventOptions, OrderBy, Predicate, UdmEvent,
    UdmEventStore,
};

use super::{parse_event_data, sort_events, PhyCloudConfig, PhyCloudStore};
use crate::aggregate::compute_aggregate;
use crate::pushdown::{extract_pushdown, PushFilter};

fn ev(source_id: &str, event_type: &str, captured_at: &str, extra: serde_json::Value) -> UdmEvent {
    let mut base = json!({
        "udm_version": "0.0.3",
        "event_id": format!("id-{source_id}-{captured_at}"),
        "event_type": event_type,
        "source_id": source_id,
        "source_type": "amr",
        "captured_at": captured_at,
    });
    if let (Some(obj), Some(add)) = (base.as_object_mut(), extra.as_object()) {
        for (k, v) in add {
            obj.insert(k.clone(), v.clone());
        }
    }
    serde_json::from_value(base).unwrap()
}

#[test]
fn events_url_appends_api_path_and_trims_slash() {
    let s = PhyCloudStore::new(PhyCloudConfig::new("http://localhost:8000/", "k"));
    assert_eq!(
        s.config().events_url(),
        "http://localhost:8000/api/v1/events"
    );
}

#[test]
fn with_session_sets_scope() {
    let cfg = PhyCloudConfig::new("http://x", "k").with_session("mission-1");
    assert_eq!(cfg.session_id.as_deref(), Some("mission-1"));
}

#[test]
fn capabilities_advertise_real_support() {
    let s = PhyCloudStore::new(PhyCloudConfig::new("http://x", "k"));
    let caps = s.capabilities();
    assert_eq!(caps.backend, "phycloud");
    assert!(caps.supports_aggregation);
    assert!(caps.supports_full_text);
    assert!(caps.supports_ordered_streaming);
    assert!(!caps.version.ends_with("-stub"));
}

#[test]
fn extract_pushdown_pulls_indexed_eq_terms() {
    let pred = Predicate::And(vec![
        Predicate::Eq {
            field: "event_type".to_owned(),
            value: json!("telemetry_periodic"),
        },
        Predicate::Eq {
            field: "source_id".to_owned(),
            value: json!("turtlebot3_tb3_0"),
        },
        // Non-indexed field: must NOT be pushed down.
        Predicate::Eq {
            field: "location/latitude".to_owned(),
            value: json!(1.0),
        },
    ]);
    let mut push = PushFilter::default();
    extract_pushdown(&pred, &mut push);
    assert_eq!(push.event_type.as_deref(), Some("telemetry_periodic"));
    assert_eq!(push.source_id.as_deref(), Some("turtlebot3_tb3_0"));
    assert_eq!(push.source_type, None);
}

#[test]
fn extract_pushdown_handles_top_level_eq() {
    let pred = Predicate::Eq {
        field: "session_id".to_owned(),
        value: json!("m-1"),
    };
    let mut push = PushFilter::default();
    extract_pushdown(&pred, &mut push);
    assert_eq!(push.session_id.as_deref(), Some("m-1"));
}

#[test]
fn apply_session_scope_overrides_predicate_session() {
    let mut push = PushFilter {
        session_id: Some("from-query".to_owned()),
        ..PushFilter::default()
    };
    push.source_id = Some("s1".to_owned());
    let mut params = Vec::new();
    push.apply(&mut params, Some("scoped-mission"));
    let session: Vec<&String> = params
        .iter()
        .filter(|(k, _)| k == "session_id")
        .map(|(_, v)| v)
        .collect();
    assert_eq!(session, vec![&"scoped-mission".to_owned()]);
}

#[test]
fn parse_event_data_extracts_payload() {
    let item = json!({
        "event_id": "id-1",
        "event_data": {
            "udm_version": "0.0.3",
            "event_id": "id-1",
            "event_type": "telemetry_periodic",
            "source_id": "s1",
            "source_type": "amr",
            "captured_at": "2026-06-07T19:00:00Z",
        }
    });
    let event = parse_event_data(&item).unwrap();
    assert_eq!(event.event_id, "id-1");
    assert_eq!(event.source_id, "s1");
}

#[test]
fn parse_event_data_missing_payload_is_backend_error() {
    let err = parse_event_data(&json!({"event_id": "x"})).unwrap_err();
    assert!(matches!(err, udm_eventstore::Error::Backend(_)));
}

#[test]
fn sort_events_orders_by_captured_at() {
    let mut events = vec![
        ev("s", "t", "2026-06-07T19:00:02Z", json!({})),
        ev("s", "t", "2026-06-07T19:00:00Z", json!({})),
        ev("s", "t", "2026-06-07T19:00:01Z", json!({})),
    ];
    sort_events(&mut events, OrderBy::CapturedAtAsc);
    let asc: Vec<_> = events.iter().map(|e| e.captured_at.to_rfc3339()).collect();
    assert_eq!(asc[0], "2026-06-07T19:00:00+00:00");
    assert_eq!(asc[2], "2026-06-07T19:00:02+00:00");

    sort_events(&mut events, OrderBy::CapturedAtDesc);
    assert_eq!(
        events[0].captured_at.to_rfc3339(),
        "2026-06-07T19:00:02+00:00"
    );
}

fn agg(agg_fn: AggregateFn, field: Option<&str>, group_by: &[&str]) -> AggregateQuery {
    AggregateQuery {
        agg_fn,
        field: field.map(str::to_owned),
        group_by: group_by.iter().map(|s| (*s).to_owned()).collect(),
        time_range: None,
        predicate: Predicate::always(),
    }
}

#[test]
fn aggregate_count_groups_by_source() {
    let events = vec![
        ev("a", "t", "2026-06-07T19:00:00Z", json!({})),
        ev("a", "t", "2026-06-07T19:00:01Z", json!({})),
        ev("b", "t", "2026-06-07T19:00:02Z", json!({})),
    ];
    let res = compute_aggregate(&agg(AggregateFn::Count, None, &["source_id"]), &events).unwrap();
    assert_eq!(res.buckets.len(), 2);
    let by_key: std::collections::HashMap<_, _> = res
        .buckets
        .iter()
        .map(|b| (b.key.get("source_id").unwrap().as_str().unwrap(), b.value))
        .collect();
    assert!((by_key["a"] - 2.0).abs() < 1e-9);
    assert!((by_key["b"] - 1.0).abs() < 1e-9);
}

#[test]
fn aggregate_avg_min_max_over_numeric_field() {
    let events = vec![
        ev(
            "a",
            "t",
            "2026-06-07T19:00:00Z",
            json!({"power": {"battery": {"state_of_charge_pct": 80.0}}}),
        ),
        ev(
            "a",
            "t",
            "2026-06-07T19:00:01Z",
            json!({"power": {"battery": {"state_of_charge_pct": 60.0}}}),
        ),
    ];
    let field = "power/battery/state_of_charge_pct";
    let avg = compute_aggregate(&agg(AggregateFn::Avg, Some(field), &[]), &events).unwrap();
    assert!((avg.buckets[0].value - 70.0).abs() < 1e-9);
    let min = compute_aggregate(&agg(AggregateFn::Min, Some(field), &[]), &events).unwrap();
    assert!((min.buckets[0].value - 60.0).abs() < 1e-9);
    let max = compute_aggregate(&agg(AggregateFn::Max, Some(field), &[]), &events).unwrap();
    assert!((max.buckets[0].value - 80.0).abs() < 1e-9);
}

#[test]
fn aggregate_non_count_without_field_is_invalid() {
    let err = compute_aggregate(&agg(AggregateFn::Sum, None, &[]), &[]).unwrap_err();
    assert!(matches!(err, udm_eventstore::Error::InvalidQuery(_)));
}

#[tokio::test]
async fn get_event_against_dead_endpoint_is_unavailable() {
    // Port 1 is never listening; connect fails fast -> Unavailable.
    let store = PhyCloudStore::new(PhyCloudConfig {
        endpoint: "http://127.0.0.1:1".to_owned(),
        api_key: "k".to_owned(),
        tenant_id: None,
        session_id: None,
        timeout_secs: 2,
    });
    let id: EventId = "any".to_owned();
    let err = store
        .get_event(&id, &GetEventOptions::default())
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        udm_eventstore::Error::Unavailable(_) | udm_eventstore::Error::Backend(_)
    ));
}
