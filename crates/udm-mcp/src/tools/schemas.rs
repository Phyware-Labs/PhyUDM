//! Embedded UDM JSON Schema for the introspection tools
//! (`explain_field`, `list_event_types`, `validate_udm_event`).
//!
//! Mirrors `udm-cli::schemas` but exposes only what the MCP server
//! needs.

use std::collections::BTreeMap;

use serde_json::Value;

pub const DEFAULT_VERSION: &str = "0.0.3";

const V003_EVENT: &str = include_str!("../../../../schemas/v0.0.3/event.schema.json");
const V003_ENVELOPE: &str = include_str!("../../../../schemas/v0.0.3/envelope.schema.json");
const V003_OBJECT_REF: &str = include_str!("../../../../schemas/v0.0.3/object_ref.schema.json");

macro_rules! embed_domain {
    ($num:expr, $slug:literal) => {
        (
            $slug,
            include_str!(concat!(
                "../../../../schemas/v0.0.3/domains/",
                stringify!($num),
                "-",
                $slug,
                ".schema.json"
            )),
        )
    };
}

const V003_DOMAINS: &[(&str, &str)] = &[
    embed_domain!(01, "identity"),
    embed_domain!(02, "location"),
    embed_domain!(03, "motion"),
    embed_domain!(04, "power"),
    embed_domain!(05, "operational"),
    embed_domain!(06, "navigation"),
    embed_domain!(07, "perception"),
    embed_domain!(08, "safety"),
    embed_domain!(09, "actuators"),
    embed_domain!(10, "communication"),
    embed_domain!(11, "compute"),
    embed_domain!(12, "ai"),
    embed_domain!(13, "maintenance"),
    embed_domain!(14, "context"),
    embed_domain!(15, "payload"),
    embed_domain!(16, "manipulation"),
    embed_domain!(17, "hri"),
    embed_domain!(18, "coordination"),
    embed_domain!(19, "simulation"),
    embed_domain!(20, "thermal"),
    embed_domain!(21, "audio"),
    embed_domain!(22, "environment-interaction"),
    embed_domain!(23, "compliance"),
];

/// Owned bundle of every schema artifact for a given version.
pub struct Bundle {
    pub version: String,
    pub event: Value,
    pub envelope: Value,
    pub object_ref: Value,
    pub domains: BTreeMap<String, Value>,
}

pub fn load(version: &str) -> Result<Bundle, String> {
    if version != DEFAULT_VERSION {
        return Err(format!(
            "unknown schema version {version:?}; udm-mcp ships v{DEFAULT_VERSION}"
        ));
    }
    let mut domains = BTreeMap::new();
    for (slug, body) in V003_DOMAINS {
        let key = (*slug).replace('-', "_");
        let value: Value = serde_json::from_str(body)
            .map_err(|err| format!("invalid embedded domain {slug}: {err}"))?;
        domains.insert(key, value);
    }
    Ok(Bundle {
        version: DEFAULT_VERSION.to_owned(),
        event: serde_json::from_str(V003_EVENT)
            .map_err(|err| format!("invalid embedded event schema: {err}"))?,
        envelope: serde_json::from_str(V003_ENVELOPE)
            .map_err(|err| format!("invalid embedded envelope schema: {err}"))?,
        object_ref: serde_json::from_str(V003_OBJECT_REF)
            .map_err(|err| format!("invalid embedded object_ref schema: {err}"))?,
        domains,
    })
}

/// Extract the canonical `EventType` enum values from the embedded
/// envelope schema (the event schema $refs envelope for envelope
/// fields). Used by `list_event_types`.
pub fn event_type_values(bundle: &Bundle) -> Vec<String> {
    bundle
        .envelope
        .pointer("/properties/event_type/enum")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}
