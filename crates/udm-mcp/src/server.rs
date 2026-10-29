//! Server implementation. Hosts the 10 MCP tools (7 data-plane + 3
//! schema-introspection) the analysis layer needs.

use std::sync::Arc;

use boon::{Compiler, SchemaIndex, Schemas};
use chrono::Utc;
use futures::StreamExt;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
};
use rmcp::{tool, tool_handler, tool_router, ErrorData, ServerHandler};
use serde_json::{json, Value};
use udm_eventstore::{
    AggregateFn, AggregateQuery, EventQuery, GetEventOptions, OrderBy, Predicate, TimeRange,
};

use crate::store::StoreHandle;
use crate::tools::compliance;
use crate::tools::filter_expr::parse_filters;
use crate::tools::requests::{
    AggregateRequest, ComplianceAuditRequest, CorrelateEventsRequest, ExplainFieldRequest,
    GetEventRequest, IncidentReconstructionRequest, ListEventTypesRequest, QueryEventsRequest,
    TimelineRequest, ValidateUdmEventRequest,
};
use crate::tools::schemas;

/// The MCP server.
///
/// Holds an [`UdmEventStore`] handle plus a cached compiled validator
/// for the embedded schemas (`validate_udm_event`).
#[derive(Clone)]
pub struct UdmAnalysisServer {
    store: StoreHandle,
    validator: Arc<EmbeddedValidator>,
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

impl UdmAnalysisServer {
    /// Build a new server wrapping `store`.
    pub fn new(store: StoreHandle) -> Self {
        let validator = Arc::new(EmbeddedValidator::compile());
        Self {
            store,
            validator,
            tool_router: Self::tool_router(),
        }
    }

    /// Expose the wrapped store's capabilities (used by `main` for
    /// startup logging).
    pub fn store_capabilities(&self) -> &udm_eventstore::StoreCapabilities {
        self.store.capabilities()
    }
}

#[tool_router]
impl UdmAnalysisServer {
    // ===================================================================
    // Data-plane tools (analysis — the primary surface)
    // ===================================================================

    /// Structured search over the event store. Use this to fetch a
    /// page of events that match a time range, source filter, and
    /// composable field predicates. Outputs JSON suitable for further
    /// reasoning. Paginate via the returned `next_cursor`.
    #[tool(
        name = "query_events",
        description = "Structured paginated search over the UDM event store. Filter expressions use the same syntax as `udm query --filter` (`field=value`, `field!=value`, `field in [a,b]`, `field contains text`, `field exists`). Paginate via `next_cursor`."
    )]
    async fn query_events(
        &self,
        Parameters(req): Parameters<QueryEventsRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let predicate =
            parse_filters(&req.filters).map_err(|e| ErrorData::invalid_params(e, None))?;
        let time_range = match req.time_range {
            None => None,
            Some(tr) => Some(
                TimeRange::new(
                    crate::util::parse_ts("from", &tr.from)?,
                    crate::util::parse_ts("to", &tr.to)?,
                )
                .map_err(crate::util::map_store_err)?,
            ),
        };
        let query = EventQuery {
            time_range,
            source_id: req.source_id,
            predicate,
            order_by: OrderBy::default(),
            limit: req.limit,
            cursor: req.cursor,
        };
        let page = self
            .store
            .query_events(&query)
            .await
            .map_err(crate::util::map_store_err)?;
        json_result(&json!({
            "events": page.events,
            "next_cursor": page.next_cursor,
            "total_estimate": page.total_estimate,
        }))
    }

    /// Fetch one UDM event by id, optionally including provenance.
    #[tool(
        name = "get_event",
        description = "Fetch one UDM event by id. Returns the full event payload; pass include_provenance to also hydrate the signing block."
    )]
    async fn get_event(
        &self,
        Parameters(req): Parameters<GetEventRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let opts = GetEventOptions {
            include_provenance: req.include_provenance,
        };
        let event = self
            .store
            .get_event(&req.event_id, &opts)
            .await
            .map_err(crate::util::map_store_err)?;
        match event {
            Some(ev) => json_result(&ev),
            None => Ok(CallToolResult::error(vec![Content::text(format!(
                "event_id {:?} not found",
                req.event_id
            ))])),
        }
    }

    /// Reconstruct the timeline for a single source across a window.
    /// Returns events in ascending captured_at order.
    #[tool(
        name = "timeline",
        description = "Time-ordered event stream for a single source across `time_range`. Returns events in ascending captured_at order. Use this to reconstruct what an autonomous system was doing."
    )]
    async fn timeline(
        &self,
        Parameters(req): Parameters<TimelineRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let start = crate::util::parse_ts("from", &req.time_range.from)?;
        let end = crate::util::parse_ts("to", &req.time_range.to)?;
        let range = TimeRange::new(start, end).map_err(crate::util::map_store_err)?;
        let mut stream = self
            .store
            .timeline(&req.source_id, range)
            .await
            .map_err(crate::util::map_store_err)?;
        let mut events = Vec::new();
        while let Some(item) = stream.next().await {
            if events.len() >= req.limit {
                break;
            }
            events.push(item.map_err(crate::util::map_store_err)?);
        }
        json_result(&json!({
            "source_id": req.source_id,
            "from": req.time_range.from,
            "to": req.time_range.to,
            "event_count": events.len(),
            "events": events,
        }))
    }

    /// Find related events across domains around a seed event.
    /// Useful for incident root-cause analysis.
    #[tool(
        name = "correlate_events",
        description = "Given a seed event id and a time window, return every event captured within that window (optionally filtered to a set of domains). Useful for incident root-cause analysis."
    )]
    async fn correlate_events(
        &self,
        Parameters(req): Parameters<CorrelateEventsRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let seed = self
            .store
            .get_event(&req.seed_event_id, &GetEventOptions::default())
            .await
            .map_err(crate::util::map_store_err)?
            .ok_or_else(|| {
                ErrorData::invalid_params(
                    format!("seed_event_id {:?} not found", req.seed_event_id),
                    None,
                )
            })?;
        let window = humantime::parse_duration(&req.window)
            .map_err(|e| ErrorData::invalid_params(format!("invalid window: {e}"), None))?;
        let half = chrono::Duration::from_std(window)
            .map_err(|e| ErrorData::invalid_params(format!("window too large: {e}"), None))?;
        let range = TimeRange::new(seed.captured_at - half, seed.captured_at + half)
            .map_err(crate::util::map_store_err)?;

        let mut predicate = Predicate::And(Vec::new());
        if !req.domains.is_empty() {
            let mut domain_checks = Vec::with_capacity(req.domains.len());
            for d in &req.domains {
                domain_checks.push(Predicate::Exists {
                    field: d.replace('-', "_"),
                });
            }
            predicate = Predicate::And(vec![predicate, Predicate::Or(domain_checks)]);
        }

        let query = EventQuery {
            time_range: Some(range),
            source_id: None,
            predicate,
            order_by: OrderBy::default(),
            limit: self.store.capabilities().max_page_size,
            cursor: None,
        };
        let page = self
            .store
            .query_events(&query)
            .await
            .map_err(crate::util::map_store_err)?;
        json_result(&json!({
            "seed_event_id": req.seed_event_id,
            "seed_captured_at": seed.captured_at,
            "window_secs": window.as_secs(),
            "domain_filter": req.domains,
            "related_event_count": page.events.len(),
            "events": page.events,
        }))
    }

    /// Compute group / fleet metrics.
    #[tool(
        name = "aggregate",
        description = "Compute fleet/group metrics over the event store. Supported aggregates: count, sum, avg, min, max. Group_by + filters scope the buckets."
    )]
    async fn aggregate(
        &self,
        Parameters(req): Parameters<AggregateRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let agg_fn = match req.agg_fn.to_ascii_lowercase().as_str() {
            "count" => AggregateFn::Count,
            "sum" => AggregateFn::Sum,
            "avg" => AggregateFn::Avg,
            "min" => AggregateFn::Min,
            "max" => AggregateFn::Max,
            other => {
                return Err(ErrorData::invalid_params(
                    format!("agg_fn must be count|sum|avg|min|max (got {other:?})"),
                    None,
                ))
            }
        };
        let predicate =
            parse_filters(&req.filters).map_err(|e| ErrorData::invalid_params(e, None))?;
        let time_range = match req.time_range {
            None => None,
            Some(tr) => Some(
                TimeRange::new(
                    crate::util::parse_ts("from", &tr.from)?,
                    crate::util::parse_ts("to", &tr.to)?,
                )
                .map_err(crate::util::map_store_err)?,
            ),
        };
        let agg = AggregateQuery {
            agg_fn,
            field: req.field,
            group_by: req.group_by,
            time_range,
            predicate,
        };
        let result = self
            .store
            .aggregate(&agg)
            .await
            .map_err(crate::util::map_store_err)?;
        json_result(&result)
    }

    /// Compliance audit over a window.
    #[tool(
        name = "compliance_audit",
        description = "Return every event bearing on a regulatory standard over a window, optionally scoped to one source. Supported standards: iso-ts-15066, iso-13482, ansi-ria-r15.06, iso-3691-4."
    )]
    async fn compliance_audit(
        &self,
        Parameters(req): Parameters<ComplianceAuditRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let std = compliance::lookup(&req.standard).ok_or_else(|| {
            ErrorData::invalid_params(
                format!(
                    "unknown standard {:?}; supported: {}",
                    req.standard,
                    compliance::STANDARDS
                        .iter()
                        .map(|s| s.id)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                None,
            )
        })?;
        let start = crate::util::parse_ts("from", &req.time_range.from)?;
        let end = crate::util::parse_ts("to", &req.time_range.to)?;
        let range = TimeRange::new(start, end).map_err(crate::util::map_store_err)?;
        let mut predicate = compliance::audit_predicate(std);
        if let Some(sid) = &req.source_id {
            predicate = Predicate::And(vec![
                predicate,
                Predicate::Eq {
                    field: "source_id".into(),
                    value: Value::String(sid.clone()),
                },
            ]);
        }
        let query = EventQuery {
            time_range: Some(range),
            source_id: req.source_id.clone(),
            predicate,
            order_by: OrderBy::default(),
            limit: self.store.capabilities().max_page_size,
            cursor: None,
        };
        let page = self
            .store
            .query_events(&query)
            .await
            .map_err(crate::util::map_store_err)?;
        json_result(&json!({
            "standard": std.id,
            "name": std.name,
            "notes": std.notes,
            "from": req.time_range.from,
            "to": req.time_range.to,
            "source_id": req.source_id,
            "matching_event_count": page.events.len(),
            "events": page.events,
        }))
    }

    /// Timeline-bracketed event bundle around a moment.
    #[tool(
        name = "incident_reconstruction",
        description = "Bracketed event bundle around a moment for root-cause analysis. Equivalent to timeline() centred on `timestamp` with `window` either side."
    )]
    async fn incident_reconstruction(
        &self,
        Parameters(req): Parameters<IncidentReconstructionRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let centre = crate::util::parse_ts("timestamp", &req.timestamp)?;
        let window = humantime::parse_duration(&req.window)
            .map_err(|e| ErrorData::invalid_params(format!("invalid window: {e}"), None))?;
        let half = chrono::Duration::from_std(window)
            .map_err(|e| ErrorData::invalid_params(format!("window too large: {e}"), None))?;
        let range =
            TimeRange::new(centre - half, centre + half).map_err(crate::util::map_store_err)?;
        let mut stream = self
            .store
            .timeline(&req.source_id, range)
            .await
            .map_err(crate::util::map_store_err)?;
        let mut events = Vec::new();
        while let Some(item) = stream.next().await {
            events.push(item.map_err(crate::util::map_store_err)?);
        }
        json_result(&json!({
            "source_id": req.source_id,
            "centre": req.timestamp,
            "window_secs": window.as_secs(),
            "event_count": events.len(),
            "events": events,
        }))
    }

    // ===================================================================
    // Schema-introspection tools (helpers for the analyst agent)
    // ===================================================================

    /// Print the spec metadata for a UDM field path.
    #[tool(
        name = "explain_field",
        description = "Print spec metadata (type / description / enum values) for a UDM field path (e.g. `safety/safety_state`). Use this to self-orient while writing queries."
    )]
    #[allow(clippy::unused_self)]
    fn explain_field(
        &self,
        Parameters(req): Parameters<ExplainFieldRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let version = req
            .schema_version
            .as_deref()
            .unwrap_or(schemas::DEFAULT_VERSION);
        let bundle = schemas::load(version).map_err(|e| ErrorData::invalid_params(e, None))?;
        let segments: Vec<&str> = req
            .path
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        if segments.is_empty() {
            return Err(ErrorData::invalid_params("path must be non-empty", None));
        }
        let first = segments[0];
        // Top-level envelope fields live in the envelope schema; domain
        // payloads live in their per-domain schemas.
        let envelope_head = bundle.envelope.pointer(&format!("/properties/{first}"));
        let event_head = bundle.event.pointer(&format!("/properties/{first}"));
        let head = envelope_head.or(event_head).ok_or_else(|| {
            ErrorData::invalid_params(format!("{first:?} is not a top-level event field"), None)
        })?;
        let mut current = head;
        let owned_first = if bundle.domains.contains_key(first) && segments.len() > 1 {
            Some(bundle.domains[first].clone())
        } else {
            None
        };
        if let Some(ref v) = owned_first {
            current = v;
        }
        for seg in &segments[1..] {
            current = current
                .pointer(&format!("/properties/{seg}"))
                .ok_or_else(|| {
                    ErrorData::invalid_params(format!("{seg:?} not found under {first:?}"), None)
                })?;
        }
        json_result(&json!({
            "path": req.path,
            "schema_version": version,
            "title": current.get("title"),
            "description": current.get("description"),
            "type": current.get("type"),
            "enum": current.get("enum"),
        }))
    }

    /// List the canonical UDM event_type enum values.
    #[tool(
        name = "list_event_types",
        description = "List the canonical UDM event_type enum values. Optional case-insensitive substring filter (e.g. `safety`, `task`)."
    )]
    #[allow(clippy::unused_self)]
    fn list_event_types(
        &self,
        Parameters(req): Parameters<ListEventTypesRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let bundle = schemas::load(schemas::DEFAULT_VERSION)
            .map_err(|e| ErrorData::invalid_params(e, None))?;
        let mut values = schemas::event_type_values(&bundle);
        if let Some(filter) = req.filter {
            let needle = filter.to_ascii_lowercase();
            values.retain(|v| v.to_ascii_lowercase().contains(&needle));
        }
        json_result(&json!({
            "schema_version": bundle.version,
            "event_types": values,
        }))
    }

    /// Validate a payload against the canonical UDM event schema.
    #[tool(
        name = "validate_udm_event",
        description = "Sanity-check a raw payload against the canonical UDM event schema (Draft 2020-12). Returns ok=true with no errors when the payload conforms, ok=false with a list of validation errors otherwise."
    )]
    fn validate_udm_event(
        &self,
        Parameters(req): Parameters<ValidateUdmEventRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let version = req
            .schema_version
            .as_deref()
            .unwrap_or(schemas::DEFAULT_VERSION);
        if version != schemas::DEFAULT_VERSION {
            return Err(ErrorData::invalid_params(
                format!(
                    "schema version {version:?} not embedded; udm-mcp ships {}",
                    schemas::DEFAULT_VERSION
                ),
                None,
            ));
        }
        match self.validator.validate(&req.payload) {
            Ok(()) => json_result(&json!({
                "ok": true,
                "schema_version": version,
                "checked_at": Utc::now(),
            })),
            Err(errs) => json_result(&json!({
                "ok": false,
                "schema_version": version,
                "checked_at": Utc::now(),
                "errors": errs,
            })),
        }
    }
}

// `tool_handler` generates `async fn call_tool`/`list_tools` bodies with no
// `.await`.
#[allow(unknown_lints, clippy::unused_async_trait_impl)]
#[tool_handler]
impl ServerHandler for UdmAnalysisServer {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.protocol_version = ProtocolVersion::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.server_info = Implementation::from_build_env();
        info.instructions = Some(
            "UDM analysis tools for autonomous-system telemetry. Use `list_event_types` and \
             `explain_field` to orient before writing queries; use `query_events`, \
             `timeline`, `correlate_events`, `aggregate`, `compliance_audit`, and \
             `incident_reconstruction` to fetch + reason over events; use \
             `validate_udm_event` to sanity-check raw payloads. Producer-side tooling \
             (generating UDM) lives in the `udm` CLI's `udm template` subcommand."
                .to_owned(),
        );
        info
    }
}

// ---------------------------------------------------------------------------
// Helpers.
// ---------------------------------------------------------------------------

fn json_result<T: serde::Serialize>(value: &T) -> Result<CallToolResult, ErrorData> {
    let text = serde_json::to_string_pretty(value)
        .map_err(|err| ErrorData::internal_error(format!("serialize result: {err}"), None))?;
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

/// Compiled embedded validator for `validate_udm_event`.
struct EmbeddedValidator {
    schemas: Schemas,
    index: SchemaIndex,
}

impl EmbeddedValidator {
    fn compile() -> Self {
        let bundle = schemas::load(schemas::DEFAULT_VERSION).expect("embedded schema");
        let mut compiler = Compiler::new();
        let mut schemas = Schemas::new();
        let mut all: Vec<(String, &Value)> = vec![
            ("event".to_owned(), &bundle.event),
            ("envelope".to_owned(), &bundle.envelope),
            ("object_ref".to_owned(), &bundle.object_ref),
        ];
        for (k, v) in &bundle.domains {
            all.push((k.clone(), v));
        }
        for (name, value) in all {
            let id = value
                .get("$id")
                .and_then(Value::as_str)
                .map_or_else(|| format!("urn:udm:{name}"), str::to_owned);
            compiler
                .add_resource(&id, value.clone())
                .expect("embedded schema add_resource");
        }
        let event_id = bundle
            .event
            .get("$id")
            .and_then(Value::as_str)
            .unwrap_or("urn:udm:event")
            .to_owned();
        let index = compiler
            .compile(&event_id, &mut schemas)
            .expect("embedded schema compile");
        Self { schemas, index }
    }

    fn validate(&self, instance: &Value) -> Result<(), Vec<String>> {
        match self.schemas.validate(instance, self.index) {
            Ok(()) => Ok(()),
            Err(err) => {
                let mut out = vec![err.to_string()];
                for cause in &err.causes {
                    out.push(format!("- {cause}"));
                }
                Err(out)
            }
        }
    }
}
