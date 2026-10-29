//! `udm audit <standard> --from --to [--source-id]`.

use std::io::Write;

use serde::Serialize;
use udm_eventstore::{EventQuery, OrderBy, Predicate, TimeRange, UdmEvent};

use crate::compliance;
use crate::output::OutputFormat;
use crate::store::StoreHandle;
use crate::{CliError, CliResult};

#[derive(Debug, Serialize)]
struct AuditOutput<'a> {
    standard: &'a str,
    name: &'a str,
    notes: &'a str,
    from: &'a str,
    to: &'a str,
    source_id: Option<&'a str>,
    matching_event_count: usize,
    events: Vec<UdmEvent>,
}

pub async fn run(
    store: &StoreHandle,
    standard: &str,
    from: &str,
    to: &str,
    source_id: Option<&str>,
    output: OutputFormat,
    mut out: impl Write,
) -> CliResult<()> {
    let std = compliance::lookup(standard).ok_or_else(|| {
        CliError::Usage(format!(
            "unknown compliance standard {standard:?}; supported: {}",
            compliance::STANDARDS
                .iter()
                .map(|s| s.id)
                .collect::<Vec<_>>()
                .join(", ")
        ))
    })?;
    let start = super::parse_ts(from)?;
    let end = super::parse_ts(to)?;
    let range = TimeRange::new(start, end).map_err(crate::CliError::EventStore)?;
    let mut predicate = compliance::audit_predicate(std);
    if let Some(sid) = source_id {
        predicate = Predicate::And(vec![
            predicate,
            Predicate::Eq {
                field: "source_id".into(),
                value: serde_json::Value::String(sid.to_owned()),
            },
        ]);
    }
    let query = EventQuery {
        time_range: Some(range),
        source_id: source_id.map(str::to_owned),
        predicate,
        order_by: OrderBy::default(),
        limit: store.capabilities().max_page_size,
        cursor: None,
    };
    let page = store.query_events(&query).await?;
    let bundle = AuditOutput {
        standard: std.id,
        name: std.name,
        notes: std.notes,
        from,
        to,
        source_id,
        matching_event_count: page.events.len(),
        events: page.events,
    };
    output.write_one(&mut out, &bundle)?;
    Ok(())
}
