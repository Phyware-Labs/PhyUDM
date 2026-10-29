//! Predicate push-down: translate the indexable subset of a
//! [`Predicate`](udm_eventstore::Predicate) into PhyCloud query-string
//! filters. Everything else is evaluated client-side by the caller.

use chrono::{DateTime, Utc};
use udm_eventstore::Predicate;

/// PhyCloud's server-side indexable filters.
#[derive(Debug, Default, Clone)]
pub struct PushFilter {
    pub source_id: Option<String>,
    pub source_type: Option<String>,
    pub event_type: Option<String>,
    pub session_id: Option<String>,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

impl PushFilter {
    /// Append the filter as `(key, value)` query-string params.
    ///
    /// `session_scope` (from [`PhyCloudConfig::session_id`](crate::PhyCloudConfig::session_id))
    /// takes precedence over any `session_id` pushed from the predicate so a
    /// mission-scoped store can never be widened by a caller's query.
    pub fn apply(&self, params: &mut Vec<(String, String)>, session_scope: Option<&str>) {
        if let Some(v) = &self.source_id {
            params.push(("source_id".to_owned(), v.clone()));
        }
        if let Some(v) = &self.source_type {
            params.push(("source_type".to_owned(), v.clone()));
        }
        if let Some(v) = &self.event_type {
            params.push(("event_type".to_owned(), v.clone()));
        }
        let session = session_scope
            .map(str::to_owned)
            .or_else(|| self.session_id.clone());
        if let Some(v) = session {
            params.push(("session_id".to_owned(), v));
        }
        if let Some(t) = self.start {
            params.push(("start_time".to_owned(), t.to_rfc3339()));
        }
        if let Some(t) = self.end {
            params.push(("end_time".to_owned(), t.to_rfc3339()));
        }
    }
}

/// Set the matching field on `push` for a top-level string-equality term.
fn set_field(field: &str, value: &str, push: &mut PushFilter) {
    match field.trim_start_matches('/') {
        "source_id" => push.source_id = Some(value.to_owned()),
        "source_type" => push.source_type = Some(value.to_owned()),
        "event_type" => push.event_type = Some(value.to_owned()),
        "session_id" => push.session_id = Some(value.to_owned()),
        _ => {}
    }
}

/// Extract the push-down-able string-equality terms from `predicate`.
///
/// Only top-level [`Predicate::Eq`] terms (or those directly under a
/// top-level [`Predicate::And`]) on indexed fields are pushed down; the
/// caller still evaluates the full predicate client-side, so a partial
/// push-down is always safe.
pub fn extract_pushdown(predicate: &Predicate, push: &mut PushFilter) {
    match predicate {
        Predicate::Eq { field, value } => {
            if let Some(s) = value.as_str() {
                set_field(field, s, push);
            }
        }
        Predicate::And(children) => {
            for child in children {
                if let Predicate::Eq { field, value } = child {
                    if let Some(s) = value.as_str() {
                        set_field(field, s, push);
                    }
                }
            }
        }
        _ => {}
    }
}
