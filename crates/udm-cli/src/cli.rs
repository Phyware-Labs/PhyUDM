//! Top-level clap definitions.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::output::OutputFormat;

/// `udm` — command-line tool for validating, exploring, and analysing
/// UDM telemetry.
#[derive(Debug, Parser)]
#[command(
    name = "udm",
    version,
    about,
    long_about = None,
    propagate_version = true,
)]
pub struct Cli {
    /// Output format. JSON-Lines is the default so output pipes cleanly
    /// into `jq`, `grep`, and LLM agents.
    #[arg(long, value_enum, global = true, default_value_t = OutputFormat::Jsonl)]
    pub output: OutputFormat,

    /// Adapter selection URL — `memory:///path/to/fixture.ndjson`,
    /// `phycloud://endpoint?token=...`, etc. Defaults to the
    /// `UDM_STORE` environment variable; if neither is set, analysis
    /// subcommands return an error.
    #[arg(long, global = true, env = "UDM_STORE")]
    pub store: Option<String>,

    /// Subcommand to run.
    #[command(subcommand)]
    pub command: Command,
}

/// One subcommand of the `udm` CLI.
#[derive(Debug, Subcommand)]
pub enum Command {
    // -------- schema / validation (CI- and author-facing) -----------
    /// Validate a JSON payload against a UDM schema version.
    Validate {
        /// JSON file to validate. Use `-` to read from stdin.
        file: PathBuf,
        /// Schema version (default: latest shipped).
        #[arg(long, default_value = crate::schemas::DEFAULT_VERSION)]
        schema_version: String,
    },

    /// Inspect UDM JSON Schema artifacts.
    Schema {
        /// Schema-related subcommand.
        #[command(subcommand)]
        cmd: SchemaCommand,
    },

    /// Print the spec text for a field path (e.g. `safety/safety_state`).
    Explain {
        /// JSON-Pointer-style field path.
        path: String,
        /// Schema version (default: latest shipped).
        #[arg(long, default_value = crate::schemas::DEFAULT_VERSION)]
        schema_version: String,
    },

    /// Run the embedded conformance suite against the canonical schema.
    Conformance {
        /// Conformance subcommand.
        #[command(subcommand)]
        cmd: ConformanceCommand,
    },

    /// Print a skeleton UDM event for hand-editing.
    Template {
        /// Source type (canonical UDM enum value, e.g. `amr`).
        #[arg(long, default_value = "amr")]
        source_type: String,
        /// Event type (canonical enum value, e.g. `telemetry_periodic`).
        #[arg(long, default_value = "telemetry_periodic")]
        event_type: String,
        /// Comma-separated list of domains to scaffold (e.g.
        /// `motion,power,safety`).
        #[arg(long, value_delimiter = ',', default_value = "identity,location")]
        domains: Vec<String>,
        /// Schema version (default: latest shipped).
        #[arg(long, default_value = crate::schemas::DEFAULT_VERSION)]
        schema_version: String,
    },

    // -------- analysis (operator- and agent-facing) -----------------
    /// Structured search over the event store.
    Query {
        /// Predicate expression (e.g.
        /// `event_type=safety_violation`, `source_id in [amr-1,amr-2]`).
        /// Repeatable; combined with implicit AND.
        #[arg(long = "filter", short = 'f', value_name = "EXPR")]
        filters: Vec<String>,
        /// Lower bound (ISO-8601). Optional.
        #[arg(long, value_name = "TIMESTAMP")]
        from: Option<String>,
        /// Upper bound (ISO-8601). Optional.
        #[arg(long, value_name = "TIMESTAMP")]
        to: Option<String>,
        /// Result page size cap.
        #[arg(long, default_value_t = 100)]
        limit: usize,
        /// Pagination cursor (echoed from a previous run's `next_cursor`).
        #[arg(long)]
        cursor: Option<String>,
        /// Optional source-id shortcut.
        #[arg(long)]
        source_id: Option<String>,
    },

    /// Fetch a single event by id.
    Get {
        /// Event id.
        event_id: String,
        /// Include provenance metadata (extra round-trip on some backends).
        #[arg(long)]
        include_provenance: bool,
    },

    /// Stream the timeline for a single source.
    Timeline {
        /// Source id.
        source_id: String,
        /// Lower bound (ISO-8601). Required.
        #[arg(long)]
        from: String,
        /// Upper bound (ISO-8601). Required.
        #[arg(long)]
        to: String,
    },

    /// Find related events across domains around a seed event.
    Correlate {
        /// Seed event id.
        event_id: String,
        /// Window (e.g. `30s`, `2m`, `1h`) around the seed.
        #[arg(long, default_value = "60s")]
        window: String,
        /// Optional comma-separated list of domains to restrict the
        /// correlation to.
        #[arg(long, value_delimiter = ',')]
        domains: Vec<String>,
    },

    /// Run a compliance audit over a window.
    Audit {
        /// Compliance standard (e.g. `iso-ts-15066`, `iso-13482`).
        standard: String,
        /// Lower bound (ISO-8601).
        #[arg(long)]
        from: String,
        /// Upper bound (ISO-8601).
        #[arg(long)]
        to: String,
        /// Optional source-id filter.
        #[arg(long)]
        source_id: Option<String>,
    },

    /// Compute group / fleet metrics.
    Aggregate {
        /// JSON-Pointer-style field path to aggregate
        /// (e.g. `power/battery/soc_pct`). Omit for `count`.
        #[arg(long)]
        field: Option<String>,
        /// Comma-separated group-by dimensions (JSON-Pointer paths).
        #[arg(long, value_delimiter = ',', default_value = "source_id")]
        by: Vec<String>,
        /// Aggregate function.
        #[arg(long, value_enum, default_value_t = AggFnArg::Count)]
        agg: AggFnArg,
        /// Lower bound (ISO-8601). Optional.
        #[arg(long)]
        from: Option<String>,
        /// Upper bound (ISO-8601). Optional.
        #[arg(long)]
        to: Option<String>,
        /// Optional predicate expression (same syntax as `udm query --filter`).
        #[arg(long = "filter", short = 'f', value_name = "EXPR")]
        filters: Vec<String>,
    },
}

/// Subcommands of `udm schema`.
#[derive(Debug, Subcommand)]
pub enum SchemaCommand {
    /// Print a JSON Schema artifact.
    #[command(disable_version_flag = true)]
    Show {
        /// Schema version.
        #[arg(default_value = crate::schemas::DEFAULT_VERSION)]
        version: String,
        /// Artifact to print: `event` (default), `envelope`,
        /// `object_ref`, or a domain key such as `safety`.
        #[arg(long, default_value = "event")]
        artifact: String,
    },
    /// Diff two schema versions (textual unified diff of the event schema).
    #[command(disable_version_flag = true)]
    Diff {
        /// Left-hand version.
        left: String,
        /// Right-hand version.
        right: String,
    },
}

/// Subcommands of `udm conformance`.
#[derive(Debug, Subcommand)]
pub enum ConformanceCommand {
    /// Run the embedded conformance suite against the canonical
    /// schema. Optional `--external` flag points the suite at an
    /// arbitrary third-party validator binary.
    Run {
        /// Schema version (default: latest shipped).
        #[arg(long, default_value = crate::schemas::DEFAULT_VERSION)]
        schema_version: String,
        /// Path to a third-party validator binary. The binary must
        /// accept a schema path as its first arg and an instance path
        /// as its second arg, exit 0 on validation success, non-zero
        /// on failure. If omitted, the CLI uses the bundled `boon`
        /// validator.
        #[arg(long)]
        external: Option<PathBuf>,
    },
}

/// Aggregate-function selection mirror of [`udm_eventstore::AggregateFn`].
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum AggFnArg {
    /// Count of matching events.
    Count,
    /// Sum of the numeric field.
    Sum,
    /// Arithmetic mean.
    Avg,
    /// Minimum.
    Min,
    /// Maximum.
    Max,
}

impl From<AggFnArg> for udm_eventstore::AggregateFn {
    fn from(value: AggFnArg) -> Self {
        match value {
            AggFnArg::Count => Self::Count,
            AggFnArg::Sum => Self::Sum,
            AggFnArg::Avg => Self::Avg,
            AggFnArg::Min => Self::Min,
            AggFnArg::Max => Self::Max,
        }
    }
}
