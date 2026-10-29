//! Tool request schemas. Each request struct derives
//! `schemars::JsonSchema` so `rmcp` can auto-generate the MCP
//! `input_schema` for the tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Time range used by multiple tools.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TimeRangeInput {
    /// Inclusive lower bound, ISO-8601 (e.g. `"2026-06-07T19:00:00Z"`).
    pub from: String,
    /// Exclusive upper bound, ISO-8601.
    pub to: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueryEventsRequest {
    /// Optional filter predicate (same expression syntax as the
    /// `udm query --filter` CLI flag). Multiple expressions are
    /// AND-ed together.
    #[serde(default)]
    pub filters: Vec<String>,
    /// Optional time-range filter.
    #[serde(default)]
    pub time_range: Option<TimeRangeInput>,
    /// Optional source-id shortcut filter.
    #[serde(default)]
    pub source_id: Option<String>,
    /// Maximum events to return in this page.
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Pagination cursor returned by a previous call's `next_cursor`.
    #[serde(default)]
    pub cursor: Option<String>,
}

fn default_limit() -> usize {
    100
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetEventRequest {
    /// UDM event identifier (UUIDv7 or backend-native id).
    pub event_id: String,
    /// Hydrate the provenance block (extra round-trip on some backends).
    #[serde(default)]
    pub include_provenance: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TimelineRequest {
    /// Source identifier (robot/sensor/fleet manager id).
    pub source_id: String,
    /// Time range to project.
    pub time_range: TimeRangeInput,
    /// Cap on the number of events returned.
    #[serde(default = "default_timeline_limit")]
    pub limit: usize,
}

fn default_timeline_limit() -> usize {
    1000
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CorrelateEventsRequest {
    /// Seed event identifier — the moment to bracket the correlation around.
    pub seed_event_id: String,
    /// Window around the seed (e.g. `"30s"`, `"2m"`, `"1h"`).
    #[serde(default = "default_window")]
    pub window: String,
    /// Restrict the correlation to events that touch any of these
    /// domains. Empty = all domains.
    #[serde(default)]
    pub domains: Vec<String>,
}

fn default_window() -> String {
    "60s".to_owned()
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AggregateRequest {
    /// Aggregate function: `count`, `sum`, `avg`, `min`, `max`.
    pub agg_fn: String,
    /// JSON-Pointer-style field path to aggregate (e.g.
    /// `"power/battery/soc_pct"`). Required for non-count.
    #[serde(default)]
    pub field: Option<String>,
    /// JSON-Pointer-style group-by dimensions.
    #[serde(default)]
    pub group_by: Vec<String>,
    /// Optional time range filter.
    #[serde(default)]
    pub time_range: Option<TimeRangeInput>,
    /// Optional filter predicates (same syntax as `query_events.filters`).
    #[serde(default)]
    pub filters: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ComplianceAuditRequest {
    /// Standard identifier (lowercase, dash-separated).
    /// Supported: `iso-ts-15066`, `iso-13482`, `ansi-ria-r15.06`, `iso-3691-4`.
    pub standard: String,
    /// Time range over which to audit.
    pub time_range: TimeRangeInput,
    /// Optional source-id filter to scope the audit to one robot.
    #[serde(default)]
    pub source_id: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IncidentReconstructionRequest {
    /// Source identifier of the autonomous system whose behaviour
    /// you're reconstructing.
    pub source_id: String,
    /// The moment to bracket the timeline around (ISO-8601).
    pub timestamp: String,
    /// Window around the moment (e.g. `"30s"`, `"2m"`, `"5m"`).
    #[serde(default = "default_incident_window")]
    pub window: String,
}

fn default_incident_window() -> String {
    "120s".to_owned()
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExplainFieldRequest {
    /// JSON-Pointer-style field path
    /// (e.g. `"safety/safety_state"`, `"identity/source_id"`,
    /// `"power/battery"`).
    pub path: String,
    /// Optional schema version override (defaults to v0.0.3).
    #[serde(default)]
    pub schema_version: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListEventTypesRequest {
    /// Optional case-insensitive substring filter
    /// (e.g. `"safety"`, `"task"`, `"telemetry"`).
    #[serde(default)]
    pub filter: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateUdmEventRequest {
    /// Raw UDM event payload to sanity-check.
    pub payload: serde_json::Value,
    /// Optional schema version override (defaults to v0.0.3).
    #[serde(default)]
    pub schema_version: Option<String>,
}
