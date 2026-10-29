//! `udm` CLI binary entry point.

use std::process::ExitCode;

use clap::Parser;
use udm_cli::{exit_code, Cli, CliError};

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match udm_cli::run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            print_chain(&err);
            ExitCode::from(exit_code(&err) as u8)
        }
    }
}

fn print_chain(err: &CliError) {
    let mut current = std::error::Error::source(err);
    while let Some(source) = current {
        eprintln!("  caused by: {source}");
        current = source.source();
    }
}
