//! Subcommand dispatcher.

mod aggregate;
mod audit;
mod conformance;
mod correlate;
mod explain;
mod filter_expr;
mod get;
mod query;
mod schema;
mod template;
mod timeline;
mod validate;

use std::time::Duration;

use crate::cli::{Cli, Command, ConformanceCommand, SchemaCommand};
use crate::store::{self, StoreHandle};
use crate::{CliError, CliResult};

/// Dispatch a parsed [`Cli`] to its subcommand handler.
pub async fn dispatch(cli: Cli) -> CliResult<()> {
    let stdout = std::io::stdout();
    let output = cli.output;
    match cli.command {
        Command::Validate {
            file,
            schema_version,
        } => validate::run(&file, &schema_version, output, stdout.lock()),
        Command::Schema { cmd } => match cmd {
            SchemaCommand::Show { version, artifact } => {
                schema::show(&version, &artifact, output, stdout.lock())
            }
            SchemaCommand::Diff { left, right } => schema::diff(&left, &right, stdout.lock()),
        },
        Command::Explain {
            path,
            schema_version,
        } => explain::run(&path, &schema_version, stdout.lock()),
        Command::Conformance { cmd } => match cmd {
            ConformanceCommand::Run {
                schema_version,
                external,
            } => conformance::run(&schema_version, external.as_deref(), stdout.lock()),
        },
        Command::Template {
            source_type,
            event_type,
            domains,
            schema_version,
        } => template::run(
            &source_type,
            &event_type,
            &domains,
            &schema_version,
            output,
            stdout.lock(),
        ),
        Command::Query {
            filters,
            from,
            to,
            limit,
            cursor,
            source_id,
        } => {
            let store = require_store(cli.store.as_deref()).await?;
            query::run(
                &store,
                &filters,
                from.as_deref(),
                to.as_deref(),
                limit,
                cursor,
                source_id,
                output,
                stdout.lock(),
            )
            .await
        }
        Command::Get {
            event_id,
            include_provenance,
        } => {
            let store = require_store(cli.store.as_deref()).await?;
            get::run(&store, &event_id, include_provenance, output, stdout.lock()).await
        }
        Command::Timeline {
            source_id,
            from,
            to,
        } => {
            let store = require_store(cli.store.as_deref()).await?;
            timeline::run(&store, &source_id, &from, &to, output, stdout.lock()).await
        }
        Command::Correlate {
            event_id,
            window,
            domains,
        } => {
            let store = require_store(cli.store.as_deref()).await?;
            let window = parse_duration(&window)?;
            correlate::run(&store, &event_id, window, &domains, output, stdout.lock()).await
        }
        Command::Audit {
            standard,
            from,
            to,
            source_id,
        } => {
            let store = require_store(cli.store.as_deref()).await?;
            audit::run(
                &store,
                &standard,
                &from,
                &to,
                source_id.as_deref(),
                output,
                stdout.lock(),
            )
            .await
        }
        Command::Aggregate {
            field,
            by,
            agg,
            from,
            to,
            filters,
        } => {
            let store = require_store(cli.store.as_deref()).await?;
            aggregate::run(
                &store,
                field.as_deref(),
                &by,
                agg.into(),
                from.as_deref(),
                to.as_deref(),
                &filters,
                output,
                stdout.lock(),
            )
            .await
        }
    }
}

async fn require_store(url: Option<&str>) -> CliResult<StoreHandle> {
    let url = url.ok_or_else(|| {
        CliError::Usage(
            "analysis subcommands require --store URL or UDM_STORE env var \
             (e.g. memory:///path/to/fixture.ndjson)"
                .to_owned(),
        )
    })?;
    store::from_url(url).await
}

/// Parse a human-readable duration (`30s`, `2m`, `1h`).
pub fn parse_duration(s: &str) -> CliResult<Duration> {
    humantime::parse_duration(s)
        .map_err(|err| CliError::Usage(format!("invalid duration {s:?}: {err}")))
}

/// Parse an ISO-8601 timestamp into a UTC `DateTime`.
pub fn parse_ts(s: &str) -> CliResult<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|err| CliError::Usage(format!("invalid ISO-8601 timestamp {s:?}: {err}")))
}
