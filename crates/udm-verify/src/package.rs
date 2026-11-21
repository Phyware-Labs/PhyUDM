//! Evidence packages (spec `evidence-package.md`).
//!
//! A package is NDJSON: a header line holding the receiver's signed
//! manifest, then one record per event, in the manifest's order:
//!
//! ```text
//! {"type":"udm.evidence-package.v1","manifest":{…}}
//! {"type":"event","event_id":"…","sequence_number":17,"content_hash":"…","previous_hash":"…","event":{…},"received":{…}}
//! ```
//!
//! [`verify_package`] checks the manifest's signature and Merkle root, that
//! the records are the events the manifest lists, every record's content
//! hash and chain link, and, for each event kept as received, the
//! producer's signature, independently of the receiver's verdict.

use std::collections::BTreeMap;
use std::io::BufRead;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::canonical::{canonical_json, content_hash};
use crate::event::{SignatureStatus, Verifier};
use crate::hex::{self, Hash};
use crate::keys::{Es256Key, KeyEntry, KeyMaterial, Keyring, ES256};
use crate::merkle::{leaf_hash, legacy_sorted_pairs_root, merkle_root};
use crate::sequence::{Gap, SequenceInfo, SequenceTracker};

/// `type` of a package's header line.
pub const PACKAGE_TYPE: &str = "udm.evidence-package.v1";
/// `merkle_algorithm` of version 3 manifests.
pub const RFC9162: &str = "rfc9162-sha256";
/// `merkle_algorithm` of version 2 manifests.
pub const LEGACY_SORTED_PAIRS: &str = "sha256-sorted-pairs";

/// Why a package could not be read at all.
#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    /// Reading failed.
    #[error("cannot read package: {0}")]
    Io(#[from] std::io::Error),
    /// A line is not JSON.
    #[error("line {line}: not JSON: {source}")]
    Json {
        /// 1-based line number.
        line: usize,
        /// The parse error.
        source: serde_json::Error,
    },
    /// The first line is not a package header.
    #[error("not an evidence package: the first line must be {{\"type\":\"{PACKAGE_TYPE}\",\"manifest\":{{…}}}}")]
    NotAPackage,
}

/// One event record of a package.
#[derive(Debug, Clone)]
pub struct Record {
    /// 1-based line number in the package.
    pub line: usize,
    /// The record as read.
    pub value: Value,
}

/// A parsed package.
#[derive(Debug, Clone)]
pub struct Package {
    /// The signed manifest.
    pub manifest: Value,
    /// The event records, in file order.
    pub records: Vec<Record>,
}

/// Read a package from NDJSON.
pub fn read_package(reader: impl BufRead) -> Result<Package, PackageError> {
    let mut manifest = None;
    let mut records = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        let line_no = i + 1;
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line).map_err(|source| PackageError::Json {
            line: line_no,
            source,
        })?;
        if manifest.is_none() {
            if value.get("type").and_then(Value::as_str) != Some(PACKAGE_TYPE) {
                return Err(PackageError::NotAPackage);
            }
            manifest = Some(
                value
                    .get("manifest")
                    .cloned()
                    .ok_or(PackageError::NotAPackage)?,
            );
            continue;
        }
        records.push(Record {
            line: line_no,
            value,
        });
    }
    Ok(Package {
        manifest: manifest.ok_or(PackageError::NotAPackage)?,
        records,
    })
}

/// The `merkle_root` of a version 3 manifest: the RFC 9162 root over the
/// listed events' content hashes, in order, each leaf being
/// `SHA-256(0x00 ‖ content hash bytes)`. `None` if a hash is not 64 hex
/// characters.
pub fn manifest_root(content_hashes: &[String]) -> Option<String> {
    let leaves: Option<Vec<Hash>> = content_hashes
        .iter()
        .map(|h| hex::hash(h).map(|raw| leaf_hash(&raw)))
        .collect();
    leaves.map(|l| hex::encode(&merkle_root(&l)))
}

/// The `received_hash` of an event as received: lowercase hex SHA-256 of
/// its canonical JSON, signature included.
pub fn received_hash(received: &Value) -> String {
    hex::encode(&Sha256::digest(canonical_json(received).as_bytes()))
}

/// What the caller brings to a verification.
#[derive(Debug, Clone, Default)]
pub struct PackageOptions {
    /// Manifest signing keys the caller trusts (pinned). A manifest signed
    /// by any other key can be intact, but is not [`Verdict::Verified`].
    pub trusted_manifest_keys: Vec<Es256Key>,
    /// Device keys the caller supplies, used in addition to (and in
    /// preference to) the keys listed in the manifest.
    pub device_keys: Keyring,
}

/// The overall result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// Every check passed and the manifest is signed by a pinned key.
    Verified,
    /// Every check passed, but the manifest's signing key was not pinned
    /// (or not supplied): the package is consistent, but who signed it is
    /// unconfirmed.
    IntactUnpinned,
    /// Every check passed, but the manifest is not signed.
    IntactUnsigned,
    /// At least one check failed: see the problems.
    Failed,
}

impl Verdict {
    /// The verdict as a string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::IntactUnpinned => "intact_unpinned",
            Self::IntactUnsigned => "intact_unsigned",
            Self::Failed => "failed",
        }
    }
}

/// How serious a problem is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Worth knowing; does not fail the package.
    Warning,
    /// The package is not what it claims to be.
    Error,
}

/// One finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Problem {
    /// How serious it is.
    pub severity: Severity,
    /// A stable identifier, e.g. `content_hash_mismatch`.
    pub kind: &'static str,
    /// What was found.
    pub message: String,
    /// The event concerned, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// The package line concerned, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

/// The manifest checks.
#[derive(Debug, Clone, Serialize)]
pub struct ManifestReport {
    /// `manifest_version`.
    pub version: Option<i64>,
    /// `valid`, `invalid`, `unchecked` (key not supplied), `unsigned` or
    /// `unsupported_algorithm`.
    pub signature: &'static str,
    /// The key id the signature names.
    pub signer_key_id: Option<String>,
    /// Whether that key is one the caller pinned.
    pub signer_pinned: bool,
    /// `merkle_algorithm`.
    pub merkle_algorithm: Option<String>,
    /// Whether the recomputed Merkle root equals `merkle_root`.
    pub merkle_root_matches: bool,
    /// Device keys the manifest lists.
    pub keys_listed: usize,
}

/// The event checks.
#[derive(Debug, Clone, Default, Serialize)]
pub struct EventsReport {
    /// Records in the package.
    pub records: usize,
    /// Records whose content hash was recomputed and matched.
    pub content_hashes_matched: usize,
    /// Adjacent records (consecutive sequence numbers) whose chain link
    /// was checked.
    pub chain_links_checked: usize,
    /// Places where the package skips sequence numbers, so the chain link
    /// cannot be checked (e.g. the locker's scope excluded events).
    pub chain_discontinuities: usize,
    /// Events kept as received, whose producer signature was re-checked.
    pub signatures_rechecked: usize,
    /// Signed events without an as-received copy; only the receiver's
    /// verdict is available for them.
    pub signatures_not_recheckable: usize,
    /// Events the receiver recorded as unsigned and did not keep as
    /// received: there is no signature to re-check.
    pub unsigned_without_copy: usize,
    /// Re-checked signature outcomes.
    pub signatures: BTreeMap<&'static str, usize>,
    /// Sequence classifications of the re-checked events.
    pub sequence: BTreeMap<&'static str, usize>,
    /// Sequence gaps still open at the end of the package, by source.
    pub open_gaps: BTreeMap<String, Vec<Gap>>,
}

/// Result of [`verify_package`].
#[derive(Debug, Clone, Serialize)]
pub struct PackageReport {
    /// The overall result.
    pub verdict: Verdict,
    /// The manifest checks.
    pub manifest: ManifestReport,
    /// The event checks.
    pub events: EventsReport,
    /// Everything found, errors first.
    pub problems: Vec<Problem>,
}

impl PackageReport {
    /// Problems of `severity`.
    pub fn count(&self, severity: Severity) -> usize {
        self.problems
            .iter()
            .filter(|p| p.severity == severity)
            .count()
    }
}

struct Findings(Vec<Problem>);

impl Findings {
    fn add(&mut self, severity: Severity, kind: &'static str, message: impl Into<String>) {
        self.0.push(Problem {
            severity,
            kind,
            message: message.into(),
            event_id: None,
            line: None,
        });
    }

    fn at(
        &mut self,
        severity: Severity,
        kind: &'static str,
        message: impl Into<String>,
        event_id: Option<&str>,
        line: usize,
    ) {
        self.0.push(Problem {
            severity,
            kind,
            message: message.into(),
            event_id: event_id.map(str::to_string),
            line: Some(line),
        });
    }

    fn has_errors(&self) -> bool {
        self.0.iter().any(|p| p.severity == Severity::Error)
    }
}

/// Verify a package.
pub fn verify_package(package: &Package, options: &PackageOptions) -> PackageReport {
    let mut findings = Findings(Vec::new());
    let manifest = &package.manifest;
    let entries: Vec<&Value> = manifest
        .get("events")
        .and_then(Value::as_array)
        .map(|a| a.iter().collect())
        .unwrap_or_default();

    let mut report = check_manifest(manifest, &entries, options, &mut findings);
    let keyring = device_keyring(manifest, options, &mut findings);
    report.keys_listed = manifest
        .get("keys")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let events = check_records(package, &entries, &keyring, &mut findings);

    let verdict = if findings.has_errors() {
        Verdict::Failed
    } else {
        match (report.signature, report.signer_pinned) {
            ("valid", true) => Verdict::Verified,
            ("valid" | "unchecked", _) => Verdict::IntactUnpinned,
            _ => Verdict::IntactUnsigned,
        }
    };
    let mut problems = findings.0;
    problems.sort_by_key(|p| std::cmp::Reverse(p.severity));
    PackageReport {
        verdict,
        manifest: report,
        events,
        problems,
    }
}

fn check_manifest(
    manifest: &Value,
    entries: &[&Value],
    options: &PackageOptions,
    findings: &mut Findings,
) -> ManifestReport {
    use Severity::{Error, Warning};
    let str_field = |v: &Value, name: &str| v.get(name).and_then(Value::as_str).map(str::to_string);
    let version = manifest.get("manifest_version").and_then(Value::as_i64);

    // Signature over the canonical manifest without `signature`.
    let mut body = manifest.clone();
    let signature = body
        .as_object_mut()
        .and_then(|m| m.remove("signature"))
        .filter(|s| !s.is_null());
    let signer_key_id = signature.as_ref().and_then(|s| str_field(s, "key_id"));
    let mut signer_pinned = false;
    let signature_status = match &signature {
        None => {
            findings.add(Warning, "manifest_unsigned", "the manifest is not signed");
            "unsigned"
        }
        Some(sig) if str_field(sig, "algorithm").as_deref() != Some(ES256) => {
            findings.add(
                Error,
                "manifest_signature_unsupported",
                format!(
                    "unsupported manifest signature algorithm {:?}",
                    str_field(sig, "algorithm")
                ),
            );
            "unsupported_algorithm"
        }
        Some(sig) => {
            let value = str_field(sig, "value").unwrap_or_default();
            let message = canonical_json(&body);
            let pinned = options
                .trusted_manifest_keys
                .iter()
                .find(|k| Some(k.key_id()) == signer_key_id.as_deref());
            let embedded = manifest_key_from(manifest, signer_key_id.as_deref());
            match pinned.or(embedded.as_ref()) {
                Some(key) if key.verify(message.as_bytes(), &value) => {
                    signer_pinned = pinned.is_some();
                    if !signer_pinned {
                        findings.add(
                            Warning,
                            "manifest_key_not_pinned",
                            format!(
                                "the manifest is signed by key {}, which was not pinned: pass the receiver's published key to confirm who signed it",
                                signer_key_id.as_deref().unwrap_or("?")
                            ),
                        );
                    }
                    "valid"
                }
                Some(_) => {
                    findings.add(
                        Error,
                        "manifest_signature_invalid",
                        "the manifest signature does not match its contents",
                    );
                    "invalid"
                }
                None => {
                    findings.add(
                        Warning,
                        "manifest_key_unknown",
                        format!(
                            "the manifest is signed by key {}, which was not supplied: pass the receiver's published key to check it",
                            signer_key_id.as_deref().unwrap_or("?")
                        ),
                    );
                    "unchecked"
                }
            }
        }
    };

    // With keys pinned, a manifest not validly signed by one of them is not
    // the receiver's (an invalid signature is reported above).
    if !options.trusted_manifest_keys.is_empty() && !signer_pinned && signature_status != "invalid"
    {
        findings.add(
            Error,
            "manifest_signer_not_trusted",
            "the manifest is not signed by any of the pinned keys",
        );
    }

    // Event count.
    let count = manifest.get("event_count").and_then(Value::as_u64);
    if count != Some(entries.len() as u64) {
        findings.add(
            Error,
            "event_count_mismatch",
            format!(
                "the manifest says {count:?} events but lists {}",
                entries.len()
            ),
        );
    }

    // Merkle root over the listed content hashes.
    let hashes: Vec<String> = entries
        .iter()
        .map(|e| str_field(e, "content_hash").unwrap_or_default())
        .collect();
    let algorithm = str_field(manifest, "merkle_algorithm");
    let stored_root = str_field(manifest, "merkle_root");
    let computed = match algorithm.as_deref() {
        Some(RFC9162) => manifest_root(&hashes),
        Some(LEGACY_SORTED_PAIRS) => {
            findings.add(
                Warning,
                "legacy_merkle_tree",
                "the manifest uses the version 2 sorted-pairs tree; version 3 manifests use an RFC 9162 root",
            );
            Some(legacy_sorted_pairs_root(&hashes))
        }
        other => {
            findings.add(
                Error,
                "merkle_algorithm_unknown",
                format!("unknown merkle_algorithm {other:?}"),
            );
            None
        }
    };
    let merkle_root_matches = computed.is_some() && computed == stored_root;
    if computed.is_some() && !merkle_root_matches {
        findings.add(
            Error,
            "merkle_root_mismatch",
            "the Merkle root does not match the events the manifest lists",
        );
    }

    ManifestReport {
        version,
        signature: signature_status,
        signer_key_id,
        signer_pinned,
        merkle_algorithm: algorithm,
        merkle_root_matches,
        keys_listed: 0,
    }
}

/// The manifest's own signing key, when the manifest embeds it (as
/// `signing_key`: `{key_id, public_key_pem}`) and it matches `key_id`.
fn manifest_key_from(manifest: &Value, key_id: Option<&str>) -> Option<Es256Key> {
    let pem = manifest
        .get("signing_key")
        .and_then(|k| k.get("public_key_pem"))
        .and_then(Value::as_str)?;
    Es256Key::from_pem(pem)
        .ok()
        .filter(|k| Some(k.key_id()) == key_id)
}

fn device_keyring(manifest: &Value, options: &PackageOptions, findings: &mut Findings) -> Keyring {
    let mut keyring = Keyring::new();
    let time = |v: &Value, name: &str| {
        v.get(name)
            .and_then(Value::as_str)
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|t| t.with_timezone(&Utc))
    };
    for k in manifest
        .get("keys")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let listed_id = k.get("key_id").and_then(Value::as_str).unwrap_or("");
        let Some(key) = k
            .get("public_key_pem")
            .and_then(Value::as_str)
            .and_then(|pem| Es256Key::from_pem(pem).ok())
        else {
            findings.add(
                Severity::Error,
                "key_unreadable",
                format!("listed key {listed_id} is not a P-256 public key"),
            );
            continue;
        };
        if key.key_id() != listed_id {
            findings.add(
                Severity::Error,
                "key_id_mismatch",
                format!("listed key {listed_id} has id {}", key.key_id()),
            );
            continue;
        }
        let id = key.key_id().to_string();
        keyring.insert(
            id,
            KeyEntry {
                material: KeyMaterial::Es256(key),
                source_id: k
                    .get("source_id")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                binding: k.get("binding").and_then(Value::as_str).map(str::to_string),
                retired_at: time(k, "retired_at"),
                revoked_at: time(k, "revoked_at"),
            },
        );
    }
    // Caller-supplied keys take precedence.
    let mut merged = options.device_keys.clone();
    for (id, entry) in keyring.entries() {
        if !merged.contains(id) {
            merged.insert(id.clone(), entry.clone());
        }
    }
    merged
}

fn check_records(
    package: &Package,
    entries: &[&Value],
    keyring: &Keyring,
    findings: &mut Findings,
) -> EventsReport {
    use Severity::{Error, Warning};
    let mut report = EventsReport {
        records: package.records.len(),
        ..EventsReport::default()
    };
    let by_id: BTreeMap<&str, &Value> = entries
        .iter()
        .filter_map(|e| Some((e.get("event_id")?.as_str()?, *e)))
        .collect();
    let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    let mut verifier = Verifier::new(keyring);
    let mut tracker = SequenceTracker::new();
    let mut previous: Option<(i64, String)> = None;

    for record in &package.records {
        let r = &record.value;
        let line = record.line;
        let s = |v: &Value, name: &str| v.get(name).and_then(Value::as_str).map(str::to_string);
        let event_id = s(r, "event_id");
        let id = event_id.as_deref();
        if r.get("type").and_then(Value::as_str) != Some("event") {
            findings.at(
                Error,
                "record_malformed",
                "record type is not \"event\"",
                id,
                line,
            );
            continue;
        }
        let Some(event) = r.get("event") else {
            findings.at(Error, "record_malformed", "record has no event", id, line);
            continue;
        };
        let seq = r.get("sequence_number").and_then(Value::as_i64);
        let stored_hash = s(r, "content_hash").unwrap_or_default();

        // The record is an event the manifest lists, with the same receiver
        // sequence number and content hash.
        let entry = id.and_then(|id| by_id.get(id)).copied();
        match entry {
            None => findings.at(
                Error,
                "record_not_in_manifest",
                "the manifest does not list this event",
                id,
                line,
            ),
            Some(entry) => {
                if let Some(id) = entry.get("event_id").and_then(Value::as_str) {
                    if !seen.insert(id) {
                        findings.at(
                            Error,
                            "record_duplicated",
                            "the event appears twice",
                            Some(id),
                            line,
                        );
                    }
                }
                let same = |name: &str| entry.get(name) == r.get(name);
                if !same("sequence_number") || !same("content_hash") {
                    findings.at(
                        Error,
                        "record_differs_from_manifest",
                        "sequence number or content hash differs from the manifest's entry for this event",
                        id,
                        line,
                    );
                }
            }
        }

        // Content hash of the stored event.
        if content_hash(event) == stored_hash {
            report.content_hashes_matched += 1;
        } else {
            findings.at(
                Error,
                "content_hash_mismatch",
                "the stored event does not hash to its content_hash",
                id,
                line,
            );
        }

        // Chain link to the previous record.
        if let (Some((prev_seq, prev_hash)), Some(seq)) = (&previous, seq) {
            if seq == prev_seq + 1 {
                report.chain_links_checked += 1;
                if s(r, "previous_hash").as_deref() != Some(prev_hash.as_str()) {
                    findings.at(
                        Error,
                        "chain_link_broken",
                        format!(
                            "previous_hash does not match the content hash of sequence {prev_seq}"
                        ),
                        id,
                        line,
                    );
                }
            } else if seq <= *prev_seq {
                findings.at(
                    Error,
                    "records_out_of_order",
                    format!("sequence {seq} follows {prev_seq}"),
                    id,
                    line,
                );
            } else {
                report.chain_discontinuities += 1;
            }
        }
        if let Some(seq) = seq {
            previous = Some((seq, stored_hash));
        }

        // The producer's signature, on the event as received.
        let claimed = entry.and_then(|e| s(e, "signature_status"));
        let Some(received) = r.get("received") else {
            // An event the receiver recorded as unsigned has no signature
            // to re-check; any other needs its as-received copy.
            if matches!(claimed.as_deref(), Some("unsigned" | "missing_signature")) {
                report.unsigned_without_copy += 1;
            } else {
                report.signatures_not_recheckable += 1;
            }
            continue;
        };
        if let Some(expected) = entry.and_then(|e| s(e, "received_hash")) {
            let actual = received_hash(received);
            if actual != expected {
                findings.at(
                    Error,
                    "received_hash_mismatch",
                    "the as-received event does not match the manifest's received_hash",
                    id,
                    line,
                );
            }
        }
        for name in ["event_id", "source_id", "event_type"] {
            if received.get(name) != event.get(name) {
                findings.at(
                    Error,
                    "received_event_mismatch",
                    format!("the as-received event's {name} differs from the stored event's"),
                    id,
                    line,
                );
            }
        }
        let check = verifier.check_event(received);
        report.signatures_rechecked += 1;
        *report.signatures.entry(check.status.as_str()).or_default() += 1;
        if let Some(claimed) = claimed {
            let claimed_norm = if claimed == "missing_signature" {
                "unsigned"
            } else {
                claimed.as_str()
            };
            if claimed_norm != check.status.as_str() {
                let severity = if claimed == "verified" {
                    Error
                } else {
                    Warning
                };
                findings.at(
                    severity,
                    "signature_status_differs",
                    format!(
                        "the receiver recorded {claimed}; re-checking gives {}",
                        check.status.as_str()
                    ),
                    id,
                    line,
                );
            }
        }
        let trusted = check.status == SignatureStatus::Verified;
        let source = received
            .get("source_id")
            .and_then(Value::as_str)
            .unwrap_or("");
        let status = tracker.observe(source, &SequenceInfo::from_event(received), trusted);
        *report.sequence.entry(status.as_str()).or_default() += 1;
        if status.is_suspicious() {
            findings.at(
                Warning,
                "sequence_suspicious",
                format!("sequence check: {}", status.as_str()),
                id,
                line,
            );
        }
    }
    for (id, _) in by_id.iter().filter(|(id, _)| !seen.contains(*id)) {
        findings.add(
            Error,
            "event_missing",
            format!("the manifest lists event {id}, but the package does not contain it"),
        );
    }
    report.open_gaps = tracker
        .open_gaps()
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
    if report.signatures_not_recheckable > 0 {
        findings.add(
            Warning,
            "signatures_not_recheckable",
            format!(
                "{} event(s) were not kept as received, so only the receiver's signature verdict is available for them",
                report.signatures_not_recheckable
            ),
        );
    }
    report
}
