//! `udm verify` — check provenance without any service.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;
use udm_verify::package::{read_package, verify_package, PackageOptions, PACKAGE_TYPE};
use udm_verify::sequence::Gap;
use udm_verify::{
    Es256Key, Keyring, SequenceInfo, SequenceTracker, SignatureStatus, Verdict, Verifier,
};

use crate::output::OutputFormat;
use crate::{CliError, CliResult};

/// Arguments of `udm verify`.
pub struct Args<'a> {
    pub file: &'a Path,
    pub trust_keys: &'a [PathBuf],
    pub keys: &'a [PathBuf],
    pub hmac_key_id: Option<&'a str>,
    pub summary: bool,
}

/// Run the `verify` subcommand.
pub fn run(args: &Args<'_>, output: OutputFormat, mut out: impl Write) -> CliResult<()> {
    let text = read_input(args.file)?;
    let keyring = device_keys(args)?;
    let label = args.file.to_string_lossy();

    let first_line = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    let is_package = serde_json::from_str::<Value>(first_line)
        .ok()
        .and_then(|v| v.get("type").and_then(Value::as_str).map(str::to_string))
        .is_some_and(|t| t == PACKAGE_TYPE);
    if is_package {
        return verify_pkg(&text, &label, args, keyring, output, &mut out);
    }
    if !args.trust_keys.is_empty() {
        return Err(CliError::Usage(
            "--trust-key applies to evidence packages; for events, pass device keys with --key"
                .to_string(),
        ));
    }
    verify_events(&text, &label, args.summary, &keyring, output, &mut out)
}

fn read_input(file: &Path) -> CliResult<String> {
    if file.as_os_str() == "-" {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        Ok(s)
    } else {
        std::fs::read_to_string(file)
            .map_err(|e| CliError::Usage(format!("cannot read {}: {e}", file.display())))
    }
}

fn read_es256(path: &Path) -> CliResult<Es256Key> {
    let pem = std::fs::read_to_string(path)
        .map_err(|e| CliError::Usage(format!("cannot read key {}: {e}", path.display())))?;
    Es256Key::from_pem(&pem).map_err(|e| CliError::Usage(format!("{}: {e}", path.display())))
}

fn device_keys(args: &Args<'_>) -> CliResult<Keyring> {
    let mut keyring = Keyring::new();
    for path in args.keys {
        keyring.add_es256(read_es256(path)?);
    }
    if let Some(id) = args.hmac_key_id {
        let secret = std::env::var("UDM_HMAC_SECRET").map_err(|_| {
            CliError::Usage("--hmac-key-id needs the secret in UDM_HMAC_SECRET".to_string())
        })?;
        keyring.add_hmac(id, secret.into_bytes());
    }
    Ok(keyring)
}

fn verify_pkg(
    text: &str,
    label: &str,
    args: &Args<'_>,
    device_keys: Keyring,
    output: OutputFormat,
    out: &mut impl Write,
) -> CliResult<()> {
    let package =
        read_package(text.as_bytes()).map_err(|e| CliError::Validation(format!("{label}: {e}")))?;
    let options = PackageOptions {
        trusted_manifest_keys: args
            .trust_keys
            .iter()
            .map(|p| read_es256(p))
            .collect::<CliResult<_>>()?,
        device_keys,
    };
    let report = verify_package(&package, &options);
    if args.summary {
        let mut summary = serde_json::to_value(&report)?;
        if let Some(m) = summary.as_object_mut() {
            m.remove("problems");
            m.insert(
                "errors".into(),
                report.count(udm_verify::package::Severity::Error).into(),
            );
            m.insert(
                "warnings".into(),
                report.count(udm_verify::package::Severity::Warning).into(),
            );
        }
        output.write_one(&mut *out, &summary)?;
    } else {
        output.write_one(&mut *out, &report)?;
    }
    match report.verdict {
        Verdict::Verified => Ok(()),
        Verdict::Failed => Err(CliError::Validation(format!(
            "{label}: verification failed ({} errors)",
            report.count(udm_verify::package::Severity::Error)
        ))),
        Verdict::IntactUnpinned => Err(CliError::Unconfirmed(format!(
            "{label} is intact, but its signer is unconfirmed: pass the receiver's key with --trust-key"
        ))),
        Verdict::IntactUnsigned => Err(CliError::Unconfirmed(format!(
            "{label} is intact, but its manifest is not signed"
        ))),
    }
}

#[derive(Serialize)]
struct EventLine<'a> {
    line: usize,
    event_id: Option<&'a str>,
    source_id: Option<&'a str>,
    signature: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<udm_verify::SignatureMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    key_id: Option<String>,
    sequence: &'static str,
}

#[derive(Serialize)]
struct EventsSummary {
    summary: bool,
    events: usize,
    signatures: BTreeMap<&'static str, usize>,
    sequence: BTreeMap<&'static str, usize>,
    open_gaps: BTreeMap<String, Vec<Gap>>,
    ok: bool,
}

/// Statuses that mean an event is not what it claims to be.
const FAILING: [SignatureStatus; 4] = [
    SignatureStatus::InvalidSignature,
    SignatureStatus::KeySourceMismatch,
    SignatureStatus::RevokedKey,
    SignatureStatus::RetiredKey,
];

fn parse_events(text: &str, label: &str) -> CliResult<Vec<(usize, Value)>> {
    // A single (possibly pretty-printed) event, or a JSON array of events.
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        return Ok(match value {
            Value::Array(items) => items.into_iter().map(|v| (1, v)).collect(),
            other => vec![(1, other)],
        });
    }
    // Otherwise NDJSON.
    text.lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
        .map(|(i, l)| {
            serde_json::from_str(l)
                .map(|v| (i + 1, v))
                .map_err(|e| CliError::Validation(format!("{label}:{}: not JSON: {e}", i + 1)))
        })
        .collect()
}

fn verify_events(
    text: &str,
    label: &str,
    summary_only: bool,
    keyring: &Keyring,
    output: OutputFormat,
    out: &mut impl Write,
) -> CliResult<()> {
    let events = parse_events(text, label)?;
    let mut verifier = Verifier::new(keyring);
    let mut tracker = SequenceTracker::new();
    let mut signatures: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut sequence: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut lines = Vec::new();
    let mut failed = false;
    for (line, event) in &events {
        let check = verifier.check_event(event);
        let source = event.get("source_id").and_then(Value::as_str);
        let status = tracker.observe(
            source.unwrap_or(""),
            &SequenceInfo::from_event(event),
            check.status == SignatureStatus::Verified,
        );
        failed |= FAILING.contains(&check.status);
        *signatures.entry(check.status.as_str()).or_default() += 1;
        *sequence.entry(status.as_str()).or_default() += 1;
        if !summary_only {
            lines.push(EventLine {
                line: *line,
                event_id: event.get("event_id").and_then(Value::as_str),
                source_id: source,
                signature: check.status.as_str(),
                mode: check.mode,
                key_id: check.key_id,
                sequence: status.as_str(),
            });
        }
    }
    if !lines.is_empty() {
        output.write_iter(&mut *out, &lines)?;
    }
    let summary = EventsSummary {
        summary: true,
        events: events.len(),
        signatures,
        sequence,
        open_gaps: tracker
            .open_gaps()
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
        ok: !failed,
    };
    output.write_one(&mut *out, &summary)?;
    if failed {
        return Err(CliError::Validation(format!(
            "{label}: some signatures do not verify"
        )));
    }
    Ok(())
}
