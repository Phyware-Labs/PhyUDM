//! `udm explain <field-path>` — print the spec description for a
//! field path. Walks the canonical event schema's `properties` tree.

use std::io::Write;

use serde::Serialize;
use serde_json::Value;

use crate::schemas;
use crate::{CliError, CliResult};

#[derive(Debug, Serialize)]
struct ExplainOutput<'a> {
    path: &'a str,
    schema_version: &'a str,
    title: Option<String>,
    description: Option<String>,
    r#type: Option<String>,
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    enum_values: Option<Vec<Value>>,
}

/// Run the `explain` subcommand.
pub fn run(path: &str, schema_version: &str, mut out: impl Write) -> CliResult<()> {
    let bundle = schemas::load_version(schema_version)?;
    let segments: Vec<&str> = path
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    if segments.is_empty() {
        return Err(CliError::Usage("path must be non-empty".into()));
    }

    // Resolve segment 0 against the event schema, then descend into
    // the corresponding domain bundle if needed.
    let first = segments[0];
    let head_schema = bundle
        .event
        .pointer(&format!("/properties/{first}"))
        .or_else(|| bundle.event.pointer(&format!("/properties/{first}/$ref")))
        .ok_or_else(|| {
            CliError::Usage(format!(
                "{first:?} is not a top-level event field (v{schema_version})"
            ))
        })?;

    let mut current = head_schema;
    let owned_first: Option<Value> = if bundle.domains.contains_key(first) && segments.len() > 1 {
        // The event-schema entry for a domain is a $ref; hop into the
        // domain bundle for further descent.
        Some(bundle.domains[first].clone())
    } else {
        None
    };
    if let Some(ref v) = owned_first {
        current = v;
    }

    for seg in &segments[1..] {
        let next = current
            .pointer(&format!("/properties/{seg}"))
            .ok_or_else(|| {
                CliError::Usage(format!(
                    "{seg:?} not found under {first:?} (v{schema_version})"
                ))
            })?;
        // Mutating `current` to point to `next` requires `next` to
        // outlive both — same lifetime so it's fine.
        current = next;
    }

    let title = current
        .get("title")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let description = current
        .get("description")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let r#type = current
        .get("type")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let enum_values = current.get("enum").and_then(Value::as_array).cloned();

    let payload = ExplainOutput {
        path,
        schema_version,
        title,
        description,
        r#type,
        enum_values,
    };
    // Pretty by default — humans usually run `udm explain` interactively.
    let text = serde_json::to_string_pretty(&payload)?;
    writeln!(out, "{text}")?;
    Ok(())
}
