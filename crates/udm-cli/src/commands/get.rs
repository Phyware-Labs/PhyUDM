//! `udm get <event_id>`.

use std::io::Write;

use udm_eventstore::GetEventOptions;

use crate::output::OutputFormat;
use crate::store::StoreHandle;
use crate::{CliError, CliResult};

pub async fn run(
    store: &StoreHandle,
    event_id: &str,
    include_provenance: bool,
    output: OutputFormat,
    mut out: impl Write,
) -> CliResult<()> {
    let opts = GetEventOptions { include_provenance };
    let id = event_id.to_owned();
    let event = store
        .get_event(&id, &opts)
        .await?
        .ok_or_else(|| CliError::Usage(format!("event {event_id:?} not found in store")))?;
    output.write_one(&mut out, &event)?;
    Ok(())
}
