//! `udm conformance run` — run the conformance suite against the
//! embedded schema. Optional `--external` flag points at a third-party
//! validator binary.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use serde_json::Value;

use crate::commands::validate::build_validator;
use crate::{CliError, CliResult};

#[derive(Debug, Serialize)]
struct ConformanceSummary {
    schema_version: String,
    validator: String,
    valid_passed: u32,
    valid_failed: u32,
    invalid_passed: u32,
    invalid_failed: u32,
    edge_passed: u32,
    edge_failed: u32,
    legacy_drift: u32,
}

/// Run the `conformance run` subcommand.
pub fn run(schema_version: &str, external: Option<&Path>, mut out: impl Write) -> CliResult<()> {
    let conformance_root = find_conformance_root()?;

    let validator_name = external.as_ref().map_or_else(
        || "boon (embedded)".to_owned(),
        |p| format!("external:{}", p.display()),
    );

    let validate_fn: Box<dyn Fn(&Path, &Value) -> CliResult<bool>> = if let Some(bin) = external {
        let bin = bin.to_path_buf();
        let bundle_path = embed_event_schema_to_tempfile(schema_version)?;
        Box::new(move |_path, instance| run_external(&bin, &bundle_path, instance))
    } else {
        let validator = build_validator(schema_version)?;
        Box::new(move |_path, instance| Ok(validator.validate(instance).is_ok()))
    };

    let mut summary = ConformanceSummary {
        schema_version: schema_version.to_owned(),
        validator: validator_name,
        valid_passed: 0,
        valid_failed: 0,
        invalid_passed: 0,
        invalid_failed: 0,
        edge_passed: 0,
        edge_failed: 0,
        legacy_drift: 0,
    };

    process_partition(
        &conformance_root.join("valid"),
        true,
        &validate_fn,
        &mut summary.valid_passed,
        &mut summary.valid_failed,
    )?;
    process_partition(
        &conformance_root.join("edge"),
        true,
        &validate_fn,
        &mut summary.edge_passed,
        &mut summary.edge_failed,
    )?;
    process_partition(
        &conformance_root.join("invalid"),
        false,
        &validate_fn,
        &mut summary.invalid_passed,
        &mut summary.invalid_failed,
    )?;

    let legacy = conformance_root.join("legacy");
    if legacy.is_dir() {
        for entry in std::fs::read_dir(&legacy)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                summary.legacy_drift += 1;
            }
        }
    }

    let text = serde_json::to_string_pretty(&summary)?;
    writeln!(out, "{text}")?;

    if summary.valid_failed + summary.edge_failed + summary.invalid_failed > 0 {
        return Err(CliError::Validation(format!(
            "conformance run failed for v{schema_version}: \
             valid_failed={}, edge_failed={}, invalid_failed={}",
            summary.valid_failed, summary.edge_failed, summary.invalid_failed
        )));
    }

    Ok(())
}

fn process_partition(
    dir: &Path,
    expect_valid: bool,
    validate_fn: &dyn Fn(&Path, &Value) -> CliResult<bool>,
    passed: &mut u32,
    failed: &mut u32,
) -> CliResult<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let body = std::fs::read_to_string(&path)?;
        let instance: Value = serde_json::from_str(&body)?;
        let ok = validate_fn(&path, &instance)?;
        if ok == expect_valid {
            *passed += 1;
        } else {
            *failed += 1;
        }
    }
    Ok(())
}

fn embed_event_schema_to_tempfile(schema_version: &str) -> CliResult<PathBuf> {
    let bundle = crate::schemas::load_version(schema_version)?;
    let temp = tempfile_path("udm-cli-schema", "json");
    std::fs::write(&temp, serde_json::to_vec_pretty(&bundle.event)?)?;
    Ok(temp)
}

/// Return a unique-ish path under the OS temp dir without needing
/// the `tempfile` crate at runtime.
fn tempfile_path(prefix: &str, ext: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let pid = std::process::id();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    path.push(format!("{prefix}-{pid}-{now}.{ext}"));
    path
}

fn run_external(bin: &Path, schema_path: &Path, instance: &Value) -> CliResult<bool> {
    let instance_path = tempfile_path("udm-cli-instance", "json");
    std::fs::write(&instance_path, serde_json::to_vec(instance)?)?;
    let status = Command::new(bin)
        .arg(schema_path)
        .arg(&instance_path)
        .status()
        .map_err(CliError::Io)?;
    let _ = std::fs::remove_file(&instance_path);
    Ok(status.success())
}

fn find_conformance_root() -> CliResult<PathBuf> {
    // Walk upward from CWD looking for a `conformance/` dir. This lets
    // `udm conformance run` work from any subdirectory of the repo
    // checkout, while also accepting a `UDM_CONFORMANCE_ROOT` override
    // (handy when running the CLI from outside the source tree).
    if let Ok(path) = std::env::var("UDM_CONFORMANCE_ROOT") {
        return Ok(PathBuf::from(path));
    }
    let mut cwd = std::env::current_dir()?;
    loop {
        let candidate = cwd.join("conformance");
        if candidate.is_dir() {
            return Ok(candidate);
        }
        if !cwd.pop() {
            return Err(CliError::Usage(
                "could not locate conformance/ directory; \
                 set UDM_CONFORMANCE_ROOT to point at it"
                    .into(),
            ));
        }
    }
}
