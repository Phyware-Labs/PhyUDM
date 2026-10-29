//! `udm validate` — validate a JSON payload against the canonical
//! event schema for a given version.

use std::io::{self, Read, Write};
use std::path::Path;

use boon::{Compiler, Schemas};
use serde::Serialize;
use serde_json::Value;

use crate::output::OutputFormat;
use crate::schemas;
use crate::{CliError, CliResult};

#[derive(Debug, Serialize)]
struct ValidateOk<'a> {
    file: &'a str,
    schema_version: &'a str,
    ok: bool,
}

#[derive(Debug, Serialize)]
struct ValidateErr<'a> {
    file: &'a str,
    schema_version: &'a str,
    ok: bool,
    errors: Vec<String>,
}

/// Run the `validate` subcommand.
pub fn run(
    file: &Path,
    schema_version: &str,
    output: OutputFormat,
    mut out: impl Write,
) -> CliResult<()> {
    let payload = read_json(file)?;
    let validator = build_validator(schema_version)?;

    let file_label = file.to_string_lossy();
    if let Err(err) = validator.validate(&payload) {
        let errors = collect_errors(&err);
        output.write_one(
            &mut out,
            &ValidateErr {
                file: &file_label,
                schema_version,
                ok: false,
                errors,
            },
        )?;
        return Err(CliError::Validation(format!(
            "{file_label} failed validation against UDM v{schema_version}"
        )));
    }
    output.write_one(
        &mut out,
        &ValidateOk {
            file: &file_label,
            schema_version,
            ok: true,
        },
    )?;
    Ok(())
}

fn read_json(file: &Path) -> CliResult<Value> {
    let body = if file == Path::new("-") {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        std::fs::read_to_string(file)?
    };
    Ok(serde_json::from_str(&body)?)
}

/// Compile every schema in a version's bundle and return a validator
/// rooted at the event schema.
pub fn build_validator(version: &str) -> CliResult<CompiledValidator> {
    let bundle = schemas::load_version(version)?;
    let mut schemas = Schemas::new();
    let mut compiler = Compiler::new();
    compiler.enable_format_assertions();

    // Register every artifact under its $id so internal refs resolve.
    for (name, value) in bundle.iter_artifacts() {
        let id = value
            .get("$id")
            .and_then(Value::as_str)
            .map_or_else(|| format!("urn:udm:{name}"), str::to_owned);
        compiler
            .add_resource(&id, value.clone())
            .map_err(|err| CliError::Validation(format!("compile {id}: {err}")))?;
    }
    let event_id = bundle
        .event
        .get("$id")
        .and_then(Value::as_str)
        .unwrap_or("urn:udm:event")
        .to_owned();
    let index = compiler
        .compile(&event_id, &mut schemas)
        .map_err(|err| CliError::Validation(format!("compile event schema: {err}")))?;
    Ok(CompiledValidator { schemas, index })
}

/// Owned compiled validator (boon `Schemas` + the event schema's index).
pub struct CompiledValidator {
    schemas: Schemas,
    index: boon::SchemaIndex,
}

impl CompiledValidator {
    /// Validate an instance, returning a Result whose Err carries a
    /// human-readable boon `ValidationError`.
    pub fn validate<'v>(
        &self,
        instance: &'v Value,
    ) -> Result<(), Box<boon::ValidationError<'_, 'v>>> {
        self.schemas
            .validate(instance, self.index)
            .map_err(Box::new)
    }
}

fn collect_errors(err: &boon::ValidationError<'_, '_>) -> Vec<String> {
    let mut out = vec![err.to_string()];
    for cause in &err.causes {
        out.push(format!("  - {cause}"));
    }
    out
}
