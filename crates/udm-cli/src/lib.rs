//! Library entry point for `udm-cli`. The binary in `main.rs` is a
//! thin wrapper around [`run`].
//!
//! Exposed as a library so integration tests can drive the dispatcher
//! without `std::process::Command`.

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

mod cli;
mod commands;
mod compliance;
mod output;
mod schemas;
mod store;

use std::io;

pub use cli::Cli;
pub use output::OutputFormat;

/// Top-level error type returned by the CLI.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// Wraps anything the eventstore layer returns.
    #[error("eventstore: {0}")]
    EventStore(#[from] udm_eventstore::Error),
    /// User-facing argument / input error (exit code 2).
    #[error("usage: {0}")]
    Usage(String),
    /// JSON parsing failed.
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    /// Filesystem I/O failed.
    #[error("io: {0}")]
    Io(#[from] io::Error),
    /// Schema validation failed (validate / conformance commands).
    #[error("validation: {0}")]
    Validation(String),
    /// Unknown schema version requested.
    #[error(
        "unknown schema version: {0} (supported: {})",
        schemas::supported_versions_csv()
    )]
    UnknownSchemaVersion(String),
}

/// Crate-wide alias.
pub type CliResult<T> = std::result::Result<T, CliError>;

/// Convert a [`CliError`] into a Unix exit code.
pub fn exit_code(err: &CliError) -> i32 {
    match err {
        CliError::Usage(_) | CliError::UnknownSchemaVersion(_) => 2,
        _ => 1,
    }
}

/// Dispatch a parsed [`Cli`] to its subcommand handler.
pub async fn run(cli: Cli) -> CliResult<()> {
    commands::dispatch(cli).await
}
