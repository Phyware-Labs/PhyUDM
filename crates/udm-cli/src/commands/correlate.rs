//! `udm correlate <event_id> --window`.
//!
//! Strategy: fetch the seed, then query a time-bracketed window around
//! its `captured_at` and emit events grouped by source — same fleet,
//! same physical moment, different telemetry. The MCP server's
//! `correlate_events` tool uses the same algorithm.

use std::io::Write;
use std::time::Duration;

use chrono::TimeZone;
use serde::Serialize;
use udm_eventstore::{EventQuery, GetEventOptions, OrderBy, Predicate, TimeRange, UdmEvent};

use crate::output::OutputFormat;
use crate::store::StoreHandle;
use crate::{CliError, CliResult};

#[derive(Debug, Serialize)]
struct CorrelationBundle<'a> {
    seed_event_id: &'a str,
    window_secs: u64,
    related: Vec<UdmEvent>,
}

pub async fn run(
    store: &StoreHandle,
    event_id: &str,
    window: Duration,
    domains: &[String],
    output: OutputFormat,
    mut out: impl Write,
) -> CliResult<()> {
    let id = event_id.to_owned();
    let seed = store
        .get_event(&id, &GetEventOptions::default())
        .await?
        .ok_or_else(|| CliError::Usage(format!("seed event {event_id:?} not found")))?;
    let half = chrono::Duration::from_std(window)
        .map_err(|err| CliError::Usage(format!("invalid window: {err}")))?;
    let start = seed.captured_at - half;
    let end = seed.captured_at + half;
    let range = TimeRange::new(start, end).map_err(crate::CliError::EventStore)?;

    let mut predicate = Predicate::And(Vec::new());
    if !domains.is_empty() {
        let mut domain_checks = Vec::with_capacity(domains.len());
        for d in domains {
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
        limit: store.capabilities().max_page_size,
        cursor: None,
    };
    let page = store.query_events(&query).await?;
    let bundle = CorrelationBundle {
        seed_event_id: event_id,
        window_secs: window.as_secs(),
        related: page.events,
    };
    output.write_one(&mut out, &bundle)?;
    Ok(())
}

// Silence dead-code warning on the import when no test enabled.
#[allow(dead_code)]
fn _unused<Tz: TimeZone>(_tz: Tz) {}
