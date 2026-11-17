//! Evidence packages: a clean package, tampered variants, and the
//! conformance vectors built from them.
//!
//! `conformance/vectors/evidence-package.json` is generated here. ES256
//! signatures are deterministic (RFC 6979), so the file is stable; run
//! `UPDATE_VECTORS=1 cargo test -p udm-verify --test package` to rewrite it.

use p256::ecdsa::signature::Signer as _;
use p256::ecdsa::{Signature, SigningKey, VerifyingKey};
use p256::pkcs8::{EncodePublicKey, LineEnding};
use pretty_assertions::assert_eq;
use serde_json::{json, Value};
use udm_verify::canonical::{canonical_json, content_hash, leaf_payload, signing_payload};
use udm_verify::merkle::{audit_path, leaf_hash, merkle_root};
use udm_verify::package::{
    manifest_root, read_package, received_hash, verify_package, PackageOptions, Severity, Verdict,
};
use udm_verify::{hex, Es256Key};

const SOURCE: &str = "amr-001";
const EPOCH: &str = "0123456789abcdef0123456789abcdef";

fn key(seed: u8) -> (SigningKey, String, String) {
    let signing = SigningKey::from_slice(&[seed; 32]).unwrap();
    let pem = VerifyingKey::from(&signing)
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    let id = Es256Key::from_pem(&pem).unwrap().key_id().to_string();
    (signing, pem, id)
}

fn sign(key: &SigningKey, message: &str) -> String {
    let sig: Signature = key.sign(message.as_bytes());
    hex::encode(&sig.to_bytes())
}

/// An event as the producer sends it, before signing.
fn raw_event(n: u64, key_id: &str) -> Value {
    json!({
        "udm_version": "0.0.5",
        "event_id": format!("0190f5a0-0000-7000-8000-{n:012}"),
        "event_type": "telemetry",
        "source_id": SOURCE,
        "source_type": "amr",
        "captured_at": format!("2026-11-17T09:00:{:02}.500000+00:00", n),
        "motion": {"speed_mps": 0.25 * n as f64, "heading_deg": 1e-5},
        "provenance": {
            "signer": "tpm",
            "algorithm": "es256",
            "key_id": key_id,
            "signed_at": format!("2026-11-17T09:00:{:02}.600000+00:00", n),
            "sequence_num": n,
            "boot_epoch": EPOCH,
            "boot_counter": 3,
            "agent_id": "agent-amr-001",
        },
    })
}

/// What a receiver stores: the event re-serialised (timestamps normalised),
/// which is why the as-received copy is kept for signatures.
fn stored(received: &Value) -> Value {
    let mut e = received.clone();
    let t = e["captured_at"].as_str().unwrap().replace("+00:00", "Z");
    e["captured_at"] = t.into();
    e
}

/// Events 1–4 signed one by one, 5–7 as one Merkle batch, 8 unsigned.
fn received_events(device: &SigningKey, device_id: &str) -> Vec<Value> {
    let mut out = Vec::new();
    for n in 1..=4 {
        let mut e = raw_event(n, device_id);
        let sig = sign(device, &signing_payload(&e));
        e["provenance"]["signature"] = sig.into();
        out.push(e);
    }
    let batch: Vec<Value> = (5..=7).map(|n| raw_event(n, device_id)).collect();
    let leaves: Vec<hex::Hash> = batch
        .iter()
        .map(|e| leaf_hash(leaf_payload(e).as_bytes()))
        .collect();
    let statement = json!({
        "type": "udm.merkle-batch.v1",
        "batch_id": "b0000000000000000000000000000001",
        "root": hex::encode(&merkle_root(&leaves)),
        "leaf_count": 3,
        "first_sequence": 5,
        "last_sequence": 7,
        "boot_epoch": EPOCH,
        "key_id": device_id,
        "signed_at": "2026-11-17T09:00:07.900000+00:00",
    });
    let statement_sig = sign(device, &canonical_json(&statement));
    for (i, mut e) in batch.into_iter().enumerate() {
        e["provenance"]["batch"] = json!({
            "statement": statement,
            "signature": statement_sig,
            "leaf_index": i,
            "proof": audit_path(i, &leaves).unwrap().iter().map(|h| hex::encode(h)).collect::<Vec<_>>(),
        });
        out.push(e);
    }
    let mut unsigned = raw_event(8, device_id);
    unsigned.as_object_mut().unwrap().remove("provenance");
    out.push(unsigned);
    out
}

struct Built {
    manifest: Value,
    records: Vec<Value>,
}

/// A version 3 manifest and its records for `received`, sealed by
/// `manifest_key`. Receiver sequence numbers start at 101; the receiver
/// recorded each event's signature status in `claims`.
fn build(received: &[Value], device_pem: &str, device_id: &str, claims: &[&str]) -> Built {
    let mut previous = "f".repeat(64);
    let mut records = Vec::new();
    let mut entries = Vec::new();
    for (i, r) in received.iter().enumerate() {
        let event = stored(r);
        let hash = content_hash(&event);
        let seq = 101 + i as i64;
        records.push(json!({
            "type": "event",
            "event_id": r["event_id"],
            "sequence_number": seq,
            "content_hash": hash,
            "previous_hash": previous,
            "event": event,
            "received": r,
        }));
        entries.push(json!({
            "event_id": r["event_id"],
            "sequence_number": seq,
            "content_hash": hash,
            "received_hash": received_hash(r),
            "signature_status": claims[i],
            "signer_key_id": r.pointer("/provenance/key_id"),
        }));
        previous = hash;
    }
    let hashes: Vec<String> = entries
        .iter()
        .map(|e| e["content_hash"].as_str().unwrap().to_string())
        .collect();
    let (_, manifest_pem, manifest_id) = key(0x22);
    let manifest = json!({
        "manifest_version": 3,
        "issuer": "phycloud",
        "issued_at": "2026-11-17T10:00:00Z",
        "locker_id": "locker-0001",
        "name": "AMR-001 incident, 17 Nov",
        "status": "sealed",
        "scope": {"source_ids": [SOURCE]},
        "event_count": entries.len(),
        "merkle_algorithm": "rfc9162-sha256",
        "merkle_root": manifest_root(&hashes).unwrap(),
        "sealed_at": "2026-11-17T10:00:00Z",
        "events": entries,
        "absences": [],
        "keys": [{
            "key_id": device_id,
            "algorithm": "es256",
            "public_key_pem": device_pem,
            "source_id": SOURCE,
            "binding": "hardware",
        }],
        "signing_key": {"key_id": manifest_id, "public_key_pem": manifest_pem},
    });
    Built {
        manifest: seal(manifest),
        records,
    }
}

fn seal(mut manifest: Value) -> Value {
    let (signing, _, id) = key(0x22);
    manifest.as_object_mut().unwrap().remove("signature");
    let value = sign(&signing, &canonical_json(&manifest));
    manifest["signature"] = json!({"algorithm": "es256", "key_id": id, "value": value});
    manifest
}

fn ndjson(manifest: &Value, records: &[Value]) -> String {
    let mut out = serde_json::to_string(&json!({
        "type": "udm.evidence-package.v1",
        "manifest": manifest,
    }))
    .unwrap();
    out.push('\n');
    for r in records {
        out.push_str(&serde_json::to_string(r).unwrap());
        out.push('\n');
    }
    out
}

const CLAIMS: [&str; 8] = [
    "verified", "verified", "verified", "verified", "verified", "verified", "verified", "unsigned",
];

/// Every scenario: (name, package text, pin the manifest key?, expected
/// verdict, expected error kinds).
fn scenarios() -> Vec<(&'static str, String, bool, Verdict, Vec<&'static str>)> {
    let (device, device_pem, device_id) = key(0x11);
    let received = received_events(&device, &device_id);
    let clean = build(&received, &device_pem, &device_id, &CLAIMS);
    let text = |b: &Built| ndjson(&b.manifest, &b.records);
    let mut out = vec![
        ("valid", text(&clean), true, Verdict::Verified, vec![]),
        (
            "valid_unpinned",
            text(&clean),
            false,
            Verdict::IntactUnpinned,
            vec![],
        ),
    ];

    // The stored copy of an event was edited after sealing.
    let mut b = build(&received, &device_pem, &device_id, &CLAIMS);
    b.records[2]["event"]["motion"]["speed_mps"] = json!(9.5);
    out.push((
        "stored_event_edited",
        text(&b),
        true,
        Verdict::Failed,
        vec!["content_hash_mismatch"],
    ));

    // The as-received copy was edited (and its signature no longer holds).
    let mut b = build(&received, &device_pem, &device_id, &CLAIMS);
    b.records[1]["received"]["motion"]["speed_mps"] = json!(9.5);
    out.push((
        "received_event_edited",
        text(&b),
        true,
        Verdict::Failed,
        vec!["received_hash_mismatch", "signature_status_differs"],
    ));

    // An event was removed from the package but not the manifest.
    let mut b = build(&received, &device_pem, &device_id, &CLAIMS);
    b.records.remove(3);
    out.push((
        "event_removed",
        text(&b),
        true,
        Verdict::Failed,
        vec!["event_missing"],
    ));

    // An event was removed from both, the count and root recomputed, but
    // the manifest could not be re-signed.
    let mut b = build(&received, &device_pem, &device_id, &CLAIMS);
    b.records.remove(3);
    let m = b.manifest.as_object_mut().unwrap();
    let mut events = m["events"].as_array().unwrap().clone();
    events.remove(3);
    let hashes: Vec<String> = events
        .iter()
        .map(|e| e["content_hash"].as_str().unwrap().to_string())
        .collect();
    m.insert("merkle_root".into(), manifest_root(&hashes).unwrap().into());
    m.insert("event_count".into(), events.len().into());
    m.insert("events".into(), events.into());
    out.push((
        "manifest_edited",
        text(&b),
        true,
        Verdict::Failed,
        vec!["manifest_signature_invalid"],
    ));

    // A forged event signed by another key, which the receiver claimed
    // verified.
    let (rogue, _, _) = key(0x33);
    let mut forged = received;
    let mut e = raw_event(4, &device_id);
    e["motion"]["speed_mps"] = json!(0.1);
    e["provenance"]["signature"] = sign(&rogue, &signing_payload(&e)).into();
    forged[3] = e;
    let b = build(&forged, &device_pem, &device_id, &CLAIMS);
    out.push((
        "forged_device_signature",
        text(&b),
        true,
        Verdict::Failed,
        vec!["signature_status_differs"],
    ));
    out
}

fn verify(text: &str, pinned: bool) -> udm_verify::PackageReport {
    let package = read_package(text.as_bytes()).unwrap();
    let mut options = PackageOptions::default();
    if pinned {
        let (_, pem, _) = key(0x22);
        options
            .trusted_manifest_keys
            .push(Es256Key::from_pem(&pem).unwrap());
    }
    verify_package(&package, &options)
}

fn error_kinds(report: &udm_verify::PackageReport) -> Vec<&'static str> {
    let mut kinds: Vec<&'static str> = report
        .problems
        .iter()
        .filter(|p| p.severity == Severity::Error)
        .map(|p| p.kind)
        .collect();
    kinds.sort_unstable();
    kinds.dedup();
    kinds
}

#[test]
fn scenarios_give_the_expected_verdicts() {
    for (name, text, pinned, verdict, mut errors) in scenarios() {
        let report = verify(&text, pinned);
        errors.sort_unstable();
        assert_eq!(
            (name, report.verdict),
            (name, verdict),
            "{:#?}",
            report.problems
        );
        assert_eq!((name, error_kinds(&report)), (name, errors));
    }
}

#[test]
fn clean_package_report() {
    let (_, text, ..) = scenarios().remove(0);
    let report = verify(&text, true);
    assert_eq!(report.manifest.signature, "valid");
    assert!(report.manifest.signer_pinned);
    assert!(report.manifest.merkle_root_matches);
    assert_eq!(report.events.records, 8);
    assert_eq!(report.events.content_hashes_matched, 8);
    assert_eq!(report.events.chain_links_checked, 7);
    assert_eq!(report.events.signatures_rechecked, 8);
    assert_eq!(report.events.signatures.get("verified"), Some(&7));
    assert_eq!(report.events.signatures.get("unsigned"), Some(&1));
    assert_eq!(report.events.sequence.get("in_order"), Some(&6));
    assert!(report.problems.is_empty(), "{:#?}", report.problems);
}

#[test]
fn a_pinned_key_that_did_not_sign_fails() {
    let (_, text, ..) = scenarios().remove(0);
    let package = read_package(text.as_bytes()).unwrap();
    let (_, other_pem, _) = key(0x44);
    let options = PackageOptions {
        trusted_manifest_keys: vec![Es256Key::from_pem(&other_pem).unwrap()],
        ..PackageOptions::default()
    };
    let report = verify_package(&package, &options);
    assert_eq!(report.verdict, Verdict::Failed);
    assert_eq!(error_kinds(&report), vec!["manifest_signer_not_trusted"]);
}

#[test]
fn a_pinned_key_rejects_an_unsigned_manifest() {
    let (_, text, ..) = scenarios().remove(0);
    let mut package = read_package(text.as_bytes()).unwrap();
    package
        .manifest
        .as_object_mut()
        .unwrap()
        .remove("signature");
    let (_, pem, _) = key(0x22);
    let options = PackageOptions {
        trusted_manifest_keys: vec![Es256Key::from_pem(&pem).unwrap()],
        ..PackageOptions::default()
    };
    let report = verify_package(&package, &options);
    assert_eq!(report.verdict, Verdict::Failed);
    assert_eq!(error_kinds(&report), vec!["manifest_signer_not_trusted"]);
    // Without a pinned key the same manifest is intact but unsigned.
    let report = verify_package(&package, &PackageOptions::default());
    assert_eq!(report.verdict, Verdict::IntactUnsigned);
}

#[test]
fn not_a_package() {
    assert!(read_package(&b"{\"type\":\"event\"}\n"[..]).is_err());
    assert!(read_package(&b"not json\n"[..]).is_err());
    assert!(read_package(&b""[..]).is_err());
}

#[test]
fn conformance_vectors_are_current() {
    let (_, pem, id) = key(0x22);
    let doc = json!({
        "description": "Evidence package vectors (spec/evidence-package.md). Generated by crates/udm-verify/tests/package.rs; do not edit by hand.",
        "manifest_key": {"key_id": id, "public_key_pem": pem},
        "packages": scenarios().into_iter().map(|(name, text, pinned, verdict, errors)| json!({
            "name": name,
            "pin_manifest_key": pinned,
            "verdict": verdict.as_str(),
            "errors": errors,
            "package": text,
        })).collect::<Vec<_>>(),
    });
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../conformance/vectors/evidence-package.json"
    );
    let rendered = format!("{}\n", serde_json::to_string_pretty(&doc).unwrap());
    if std::env::var_os("UPDATE_VECTORS").is_some() {
        std::fs::write(path, &rendered).unwrap();
    }
    let on_disk = std::fs::read_to_string(path).unwrap_or_default();
    assert!(
        on_disk == rendered,
        "conformance/vectors/evidence-package.json is out of date: run UPDATE_VECTORS=1 cargo test -p udm-verify --test package"
    );
}
