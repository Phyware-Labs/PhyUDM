//! Query, page, predicate, and aggregate types consumed by the
//! [`UdmEventStore`](crate::UdmEventStore) trait.
//!
//! All types are backend-agnostic: predicates use JSON-Pointer-style
//! field paths so the same query expression runs against the in-memory
//! adapter, the PhyCloud adapter, or any third-party adapter without
//! translation.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::event::{EventId, SourceId, UdmEvent};

/// Half-open time interval `[start, end)` (UTC).
///
/// Adapters MUST treat `start` as inclusive and `end` as exclusive so
/// adjacent ranges tile a timeline without double-counting events at
/// the boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    /// Inclusive start (UTC).
    pub start: DateTime<Utc>,
    /// Exclusive end (UTC).
    pub end: DateTime<Utc>,
}

impl TimeRange {
    /// Build a half-open `[start, end)` range. Returns
    /// [`Error::InvalidQuery`](crate::Error::InvalidQuery) when
    /// `end <= start`.
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> crate::Result<Self> {
        if end <= start {
            return Err(crate::Error::InvalidQuery(format!(
                "TimeRange end ({end}) must be strictly after start ({start})"
            )));
        }
        Ok(Self { start, end })
    }

    /// `true` iff `ts` falls within `[start, end)`.
    pub fn contains(&self, ts: DateTime<Utc>) -> bool {
        ts >= self.start && ts < self.end
    }
}

/// Sort order used by [`EventQuery::order_by`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OrderBy {
    /// Ascending `captured_at`. Default; canonical for timeline use.
    #[default]
    CapturedAtAsc,
    /// Descending `captured_at`. Used when paging "most recent first".
    CapturedAtDesc,
    /// Insertion / `received_at` ascending (when the backend supports it).
    ReceivedAtAsc,
}

/// Field-level predicates that compose a query filter.
///
/// Paths use JSON-Pointer-style `/`-separated segments rooted at the
/// event envelope (e.g. `"event_type"`, `"safety/safety_state"`,
/// `"identity/source_id"`). Adapters that index a subset of paths MAY
/// raise [`Error::Unsupported`](crate::Error::Unsupported) for paths
/// outside their index; tools SHOULD interrogate
/// [`StoreCapabilities::indexed_paths`] first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Predicate {
    /// `field == value`. JSON-equality semantics.
    Eq {
        /// JSON-Pointer-style field path.
        field: String,
        /// Value to compare against.
        value: serde_json::Value,
    },
    /// `field != value`.
    Ne {
        /// JSON-Pointer-style field path.
        field: String,
        /// Value to compare against.
        value: serde_json::Value,
    },
    /// `field IN (values...)`.
    In {
        /// JSON-Pointer-style field path.
        field: String,
        /// Allowed values.
        values: Vec<serde_json::Value>,
    },
    /// Substring search on a string field. Case-insensitive.
    Contains {
        /// JSON-Pointer-style field path.
        field: String,
        /// Substring to look for.
        value: String,
    },
    /// Field exists and is non-null.
    Exists {
        /// JSON-Pointer-style field path.
        field: String,
    },
    /// Logical AND of sub-predicates. Empty list matches every event.
    And(Vec<Self>),
    /// Logical OR of sub-predicates. Empty list matches no events.
    Or(Vec<Self>),
}

impl Default for Predicate {
    fn default() -> Self {
        Self::And(Vec::new())
    }
}

impl Predicate {
    /// `Predicate::And(vec![])` — matches every event. Useful default.
    pub fn always() -> Self {
        Self::And(vec![])
    }

    /// Evaluate against a single event. Used by the in-memory adapter
    /// and by adapter conformance tests; production adapters typically
    /// translate predicates into native query languages instead.
    pub fn matches(&self, event: &UdmEvent) -> bool {
        let Ok(value) = serde_json::to_value(event) else {
            return false;
        };
        self.matches_value(&value)
    }

    fn matches_value(&self, root: &serde_json::Value) -> bool {
        match self {
            Self::Eq { field, value } => pointer(root, field).is_some_and(|v| v == value),
            Self::Ne { field, value } => pointer(root, field).is_none_or(|v| v != value),
            Self::In { field, values } => pointer(root, field).is_some_and(|v| values.contains(v)),
            Self::Contains { field, value } => pointer(root, field)
                .and_then(serde_json::Value::as_str)
                .is_some_and(|s| s.to_lowercase().contains(&value.to_lowercase())),
            Self::Exists { field } => pointer(root, field).is_some_and(|v| !v.is_null()),
            Self::And(children) => children.iter().all(|p| p.matches_value(root)),
            Self::Or(children) => children.iter().any(|p| p.matches_value(root)),
        }
    }
}

fn pointer<'a>(root: &'a serde_json::Value, field: &str) -> Option<&'a serde_json::Value> {
    let trimmed = field.trim_start_matches('/');
    if trimmed.is_empty() {
        return Some(root);
    }
    let pointer_str = format!("/{trimmed}");
    root.pointer(&pointer_str)
}

/// Bounded, paginated query over the event store.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventQuery {
    /// Optional time range filter (`None` means "all time").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_range: Option<TimeRange>,

    /// Optional `source_id` filter — shortcut for
    /// `Predicate::Eq{"source_id", ...}`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<SourceId>,

    /// Composable field-level predicate. Defaults to "match all".
    #[serde(default = "Predicate::always")]
    pub predicate: Predicate,

    /// Sort order (default: `captured_at` ascending).
    #[serde(default)]
    pub order_by: OrderBy,

    /// Maximum page size. Default 100; adapters MAY cap further via
    /// [`StoreCapabilities::max_page_size`].
    #[serde(default = "default_limit")]
    pub limit: usize,

    /// Opaque cursor returned by a previous page's
    /// [`EventPage::next_cursor`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

fn default_limit() -> usize {
    100
}

/// One page of an [`EventQuery`] result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPage {
    /// Events in this page, ordered per `query.order_by`.
    pub events: Vec<UdmEvent>,

    /// Opaque cursor for the next page. `None` when no more pages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,

    /// Total count when the backend can supply it cheaply; `None`
    /// otherwise. Callers MUST NOT rely on this for correctness.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_estimate: Option<u64>,
}

/// Aggregate function applied by [`UdmEventStore::aggregate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregateFn {
    /// Count of events in the bucket.
    Count,
    /// Sum of the numeric `field`.
    Sum,
    /// Arithmetic mean of the numeric `field`.
    Avg,
    /// Minimum of the numeric `field`.
    Min,
    /// Maximum of the numeric `field`.
    Max,
}

/// Aggregate query — `agg_fn(field) GROUP BY group_by[] WHERE filter`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateQuery {
    /// Aggregate function to apply.
    pub agg_fn: AggregateFn,

    /// JSON-Pointer-style field path of the numeric value to aggregate.
    /// Ignored when `agg_fn = Count`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,

    /// JSON-Pointer-style field paths to group by. Empty list means a
    /// single global aggregate.
    #[serde(default)]
    pub group_by: Vec<String>,

    /// Time range filter (same semantics as
    /// [`EventQuery::time_range`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_range: Option<TimeRange>,

    /// Field-level predicate (same semantics as
    /// [`EventQuery::predicate`]).
    #[serde(default = "Predicate::always")]
    pub predicate: Predicate,
}

/// One bucket of an aggregate result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateBucket {
    /// Group-by dimension values. Keys mirror
    /// [`AggregateQuery::group_by`] entries in order.
    pub key: BTreeMap<String, serde_json::Value>,
    /// Aggregated numeric value. `count` produces an integer; sum / avg
    /// / min / max produce a float (NaN for empty bucket — adapters
    /// MUST omit empty buckets instead).
    pub value: f64,
    /// Number of events that fed into this bucket. `count` aggregations
    /// report the same number as `value`.
    pub event_count: u64,
}

/// Result of an [`AggregateQuery`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateResult {
    /// One bucket per distinct `group_by` value combination. Empty
    /// when no events match the predicate.
    pub buckets: Vec<AggregateBucket>,
    /// Echo of the `agg_fn` for downstream tooling that needs to
    /// label the metric.
    pub agg_fn: AggregateFn,
}

/// Lookup of an event by id with optional inclusion of provenance
/// metadata — currently identical to fetching the event directly, but
/// the dedicated parameter lets adapters skip an extra round-trip when
/// the caller doesn't need provenance.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetEventOptions {
    /// When `true`, the adapter MUST hydrate
    /// [`UdmEvent::provenance`](crate::event::UdmEvent::provenance)
    /// even if it is normally lazy-loaded.
    pub include_provenance: bool,
}

impl GetEventOptions {
    /// Convenience: skip the extra round-trip for provenance.
    pub const fn minimal() -> Self {
        Self {
            include_provenance: false,
        }
    }
}

/// Self-describing capability set published by every adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreCapabilities {
    /// Human-readable backend name (`"memory"`, `"phycloud"`, …).
    pub backend: String,

    /// Backend version (or `"unknown"`).
    pub version: String,

    /// `true` when [`UdmEventStore::aggregate`] is implemented natively
    /// (vs. emulated in-memory or unsupported).
    pub supports_aggregation: bool,

    /// `true` when [`Predicate::Contains`] is implemented natively.
    pub supports_full_text: bool,

    /// `true` when [`UdmEventStore::timeline`] returns events
    /// strictly in `captured_at` order without requiring the caller
    /// to re-sort.
    pub supports_ordered_streaming: bool,

    /// Maximum value the backend will accept for
    /// [`EventQuery::limit`]. Adapters MAY clamp larger requests
    /// silently.
    pub max_page_size: usize,

    /// Field paths the backend has indexed (best-effort hint for
    /// tools that want to push down predicates).
    pub indexed_paths: Vec<String>,

    /// Identifier (e.g. UDM event id) used for opaque pagination
    /// cursors. Documentary only.
    pub cursor_scheme: String,
}

impl Default for StoreCapabilities {
    fn default() -> Self {
        Self {
            backend: "unknown".to_owned(),
            version: "unknown".to_owned(),
            supports_aggregation: false,
            supports_full_text: false,
            supports_ordered_streaming: false,
            max_page_size: 1000,
            indexed_paths: Vec::new(),
            cursor_scheme: "opaque".to_owned(),
        }
    }
}

/// Convenience constructor for the most common single-id lookup.
pub fn get_by_id(event_id: impl Into<EventId>) -> EventId {
    event_id.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ev() -> UdmEvent {
        serde_json::from_value(json!({
            "udm_version": "0.0.3",
            "event_id": "e1",
            "event_type": "safety_violation",
            "source_id": "amr-001",
            "source_type": "amr",
            "captured_at": "2026-06-07T19:00:00Z",
            "safety": {"safety_state": "emergency_stop", "is_safe": false},
        }))
        .unwrap()
    }

    #[test]
    fn predicate_eq_top_level() {
        let p = Predicate::Eq {
            field: "event_type".into(),
            value: json!("safety_violation"),
        };
        assert!(p.matches(&ev()));
    }

    #[test]
    fn predicate_eq_nested() {
        let p = Predicate::Eq {
            field: "safety/safety_state".into(),
            value: json!("emergency_stop"),
        };
        assert!(p.matches(&ev()));
    }

    #[test]
    fn predicate_in() {
        let p = Predicate::In {
            field: "source_type".into(),
            values: vec![json!("amr"), json!("agv")],
        };
        assert!(p.matches(&ev()));
    }

    #[test]
    fn predicate_contains_case_insensitive() {
        let p = Predicate::Contains {
            field: "event_type".into(),
            value: "VIOLATION".into(),
        };
        assert!(p.matches(&ev()));
    }

    #[test]
    fn predicate_exists() {
        let p = Predicate::Exists {
            field: "safety".into(),
        };
        assert!(p.matches(&ev()));
        let p = Predicate::Exists {
            field: "navigation".into(),
        };
        assert!(!p.matches(&ev()));
    }

    #[test]
    fn predicate_and_or() {
        let p = Predicate::And(vec![
            Predicate::Eq {
                field: "source_type".into(),
                value: json!("amr"),
            },
            Predicate::Or(vec![
                Predicate::Eq {
                    field: "event_type".into(),
                    value: json!("safety_violation"),
                },
                Predicate::Eq {
                    field: "event_type".into(),
                    value: json!("emergency_stop"),
                },
            ]),
        ]);
        assert!(p.matches(&ev()));
    }

    #[test]
    fn time_range_rejects_invalid_bounds() {
        let now = Utc::now();
        assert!(TimeRange::new(now, now).is_err());
        let later = now + chrono::Duration::seconds(1);
        let r = TimeRange::new(now, later).unwrap();
        assert!(r.contains(now));
        assert!(!r.contains(later));
    }
}
