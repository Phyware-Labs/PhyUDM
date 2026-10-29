//! `udm query` — paginated event search.

use std::io::Write;

use udm_eventstore::{EventQuery, OrderBy, Predicate, TimeRange};

use crate::output::OutputFormat;
use crate::store::StoreHandle;
use crate::CliResult;

use super::filter_expr;

#[allow(clippy::too_many_arguments)]
pub async fn run(
    store: &StoreHandle,
    filters: &[String],
    from: Option<&str>,
    to: Option<&str>,
    limit: usize,
    cursor: Option<String>,
    source_id: Option<String>,
    output: OutputFormat,
    mut out: impl Write,
) -> CliResult<()> {
    let predicate = filter_expr::parse(filters)?;
    let time_range = parse_time_range(from, to)?;
    let query = EventQuery {
        time_range,
        source_id,
        predicate,
        order_by: OrderBy::default(),
        limit,
        cursor,
    };
    let page = store.query_events(&query).await?;
    output.write_iter(&mut out, &page.events)?;
    if let Some(cursor) = page.next_cursor {
        writeln!(out, "# next_cursor: {cursor}")?;
    }
    Ok(())
}

pub fn parse_time_range(from: Option<&str>, to: Option<&str>) -> CliResult<Option<TimeRange>> {
    match (from, to) {
        (None, None) => Ok(None),
        (Some(f), Some(t)) => {
            let start = super::parse_ts(f)?;
            let end = super::parse_ts(t)?;
            let range = TimeRange::new(start, end).map_err(crate::CliError::EventStore)?;
            Ok(Some(range))
        }
        _ => Err(crate::CliError::Usage(
            "--from and --to must both be supplied (or neither)".into(),
        )),
    }
}

/// Helper consumed by `aggregate` (and potentially future commands).
#[allow(dead_code)]
pub fn predicate_with_source(base: Predicate, source_id: Option<&str>) -> Predicate {
    match source_id {
        None => base,
        Some(id) => Predicate::And(vec![
            base,
            Predicate::Eq {
                field: "source_id".into(),
                value: serde_json::Value::String(id.to_owned()),
            },
        ]),
    }
}
