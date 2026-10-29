//! Small shared helpers (timestamp parsing, etc.).

use chrono::{DateTime, Utc};
use rmcp::ErrorData;

/// Parse an ISO-8601 timestamp; return an MCP-friendly error when the
/// caller hands us something unparseable.
pub fn parse_ts(label: &str, s: &str) -> Result<DateTime<Utc>, ErrorData> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|err| {
            ErrorData::invalid_params(format!("{label} must be ISO-8601 (got {s:?}): {err}"), None)
        })
}

/// Map a [`udm_eventstore::Error`] into an MCP error.
pub fn map_store_err(err: udm_eventstore::Error) -> ErrorData {
    match err {
        udm_eventstore::Error::Unsupported(msg) => ErrorData::invalid_request(
            format!("backend does not support this operation: {msg}"),
            None,
        ),
        udm_eventstore::Error::InvalidQuery(msg) => ErrorData::invalid_params(msg, None),
        udm_eventstore::Error::NotFound(msg) => {
            ErrorData::invalid_params(format!("not found: {msg}"), None)
        }
        udm_eventstore::Error::Forbidden(msg) => {
            ErrorData::invalid_request(format!("forbidden: {msg}"), None)
        }
        other => ErrorData::internal_error(other.to_string(), None),
    }
}
