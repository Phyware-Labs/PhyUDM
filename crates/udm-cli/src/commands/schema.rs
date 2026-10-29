//! `udm schema show` and `udm schema diff`.

use std::io::Write;

use crate::output::OutputFormat;
use crate::schemas;
use crate::{CliError, CliResult};

/// `udm schema show`.
pub fn show(
    version: &str,
    artifact: &str,
    output: OutputFormat,
    mut out: impl Write,
) -> CliResult<()> {
    let bundle = schemas::load_version(version)?;
    let value = bundle
        .artifact(artifact)
        .ok_or_else(|| CliError::Usage(format!("unknown artifact {artifact:?} for v{version}")))?;
    output.write_one(&mut out, value)?;
    Ok(())
}

/// `udm schema diff`.
pub fn diff(left: &str, right: &str, mut out: impl Write) -> CliResult<()> {
    let lhs_bundle = schemas::load_version(left)?;
    let rhs_bundle = schemas::load_version(right)?;
    let lhs = serde_json::to_string_pretty(&lhs_bundle.event)?;
    let rhs = serde_json::to_string_pretty(&rhs_bundle.event)?;
    if lhs == rhs {
        writeln!(
            out,
            "no differences between event schemas {left} and {right}"
        )?;
        return Ok(());
    }
    let diff = similar::TextDiff::from_lines(&lhs, &rhs);
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            similar::ChangeTag::Delete => "-",
            similar::ChangeTag::Insert => "+",
            similar::ChangeTag::Equal => " ",
        };
        write!(out, "{sign}{change}")?;
    }
    Ok(())
}
