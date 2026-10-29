//! `udm aggregate`.

use std::io::Write;

use udm_eventstore::{AggregateFn, AggregateQuery};

use crate::output::OutputFormat;
use crate::store::StoreHandle;
use crate::CliResult;

use super::{filter_expr, query::parse_time_range};

#[allow(clippy::too_many_arguments)]
pub async fn run(
    store: &StoreHandle,
    field: Option<&str>,
    by: &[String],
    agg_fn: AggregateFn,
    from: Option<&str>,
    to: Option<&str>,
    filters: &[String],
    output: OutputFormat,
    mut out: impl Write,
) -> CliResult<()> {
    let predicate = filter_expr::parse(filters)?;
    let time_range = parse_time_range(from, to)?;
    let agg = AggregateQuery {
        agg_fn,
        field: field.map(str::to_owned),
        group_by: by.to_vec(),
        time_range,
        predicate,
    };
    let result = store.aggregate(&agg).await?;
    output.write_one(&mut out, &result)?;
    Ok(())
}
