//! `udm timeline <source_id> --from --to`.

use std::io::Write;

use futures::StreamExt;
use udm_eventstore::TimeRange;

use crate::output::OutputFormat;
use crate::store::StoreHandle;
use crate::CliResult;

pub async fn run(
    store: &StoreHandle,
    source_id: &str,
    from: &str,
    to: &str,
    output: OutputFormat,
    mut out: impl Write,
) -> CliResult<()> {
    let start = super::parse_ts(from)?;
    let end = super::parse_ts(to)?;
    let range = TimeRange::new(start, end).map_err(crate::CliError::EventStore)?;
    let id = source_id.to_owned();
    let mut stream = store.timeline(&id, range).await?;
    while let Some(item) = stream.next().await {
        let event = item?;
        output.write_one(&mut out, &event)?;
    }
    Ok(())
}
