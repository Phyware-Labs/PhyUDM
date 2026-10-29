//! `udm template` — print a skeleton UDM event for hand-editing.

use std::io::Write;

use serde_json::{json, Value};

use crate::output::OutputFormat;
use crate::schemas;
use crate::{CliError, CliResult};

/// Run the `template` subcommand.
pub fn run(
    source_type: &str,
    event_type: &str,
    domains: &[String],
    schema_version: &str,
    output: OutputFormat,
    mut out: impl Write,
) -> CliResult<()> {
    let bundle = schemas::load_version(schema_version)?;

    let mut payload = json!({
        "udm_version": schema_version,
        "event_id": "01940000-0000-7000-8000-000000000000",
        "event_type": event_type,
        "source_id": format!("{source_type}-001"),
        "source_type": source_type,
        "captured_at": "2026-06-07T19:00:00Z",
    });

    for domain in domains {
        let domain_key = domain.replace('-', "_");
        let schema = bundle.domains.get(&domain_key).ok_or_else(|| {
            CliError::Usage(format!(
                "unknown domain {domain:?} for v{schema_version} (try one of: {})",
                bundle
                    .domains
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })?;
        let skeleton = skeleton_for_domain(schema);
        payload[domain_key] = skeleton;
    }

    output.write_one(&mut out, &payload)?;
    Ok(())
}

fn skeleton_for_domain(schema: &Value) -> Value {
    let Some(props) = schema.get("properties").and_then(Value::as_object) else {
        return Value::Object(serde_json::Map::new());
    };
    let mut obj = serde_json::Map::new();
    for (name, def) in props {
        obj.insert(name.clone(), placeholder_for(def));
    }
    Value::Object(obj)
}

fn placeholder_for(def: &Value) -> Value {
    if let Some(enum_vals) = def.get("enum").and_then(Value::as_array) {
        return enum_vals
            .first()
            .cloned()
            .unwrap_or_else(|| Value::String("<enum>".to_owned()));
    }
    match def.get("type").and_then(Value::as_str) {
        Some("string") => Value::String("<string>".to_owned()),
        Some("number") => json!(0.0),
        Some("integer") => json!(0),
        Some("boolean") => json!(false),
        Some("array") => Value::Array(Vec::new()),
        _ => Value::Object(serde_json::Map::new()),
    }
}
