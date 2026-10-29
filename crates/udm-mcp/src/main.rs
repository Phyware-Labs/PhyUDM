//! `udm-mcp` binary entry point.
//!
//! Reads the backend selection from `--store` / `UDM_STORE` and serves
//! the [`udm_mcp::UdmAnalysisServer`] over stdio.

use std::process::ExitCode;

use clap::Parser;
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(name = "udm-mcp", version, about, long_about = None)]
struct Cli {
    /// Adapter selection URL. Examples:
    ///   `memory:///path/to/fleet.ndjson`
    ///   `phycloud://api.phycloud.example.com?token=KEY`
    #[arg(long, env = "UDM_STORE")]
    store: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    // MCP servers communicate over stdio — every log line MUST go to
    // stderr so it doesn't corrupt the JSON-RPC framing on stdout.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let cli = Cli::parse();

    let store = match udm_mcp::store_from_url(&cli.store).await {
        Ok(s) => s,
        Err(err) => {
            tracing::error!("failed to open store {:?}: {err}", cli.store);
            return ExitCode::FAILURE;
        }
    };

    let server = udm_mcp::UdmAnalysisServer::new(store);
    tracing::info!(
        backend = %server.store_capabilities().backend,
        version = %server.store_capabilities().version,
        "starting udm-mcp"
    );

    let service = match server.serve(stdio()).await {
        Ok(s) => s,
        Err(err) => {
            tracing::error!("MCP transport error: {err}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(err) = service.waiting().await {
        tracing::error!("MCP server exited with error: {err}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
