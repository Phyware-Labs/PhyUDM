//! `udm verify` end to end, driven by the conformance vectors.

use std::io::Write;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::NamedTempFile;

const PACKAGES: &str = include_str!("../../../conformance/vectors/evidence-package.json");
const BATCH: &str = include_str!("../../../conformance/vectors/merkle-batch.json");

fn temp(contents: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(contents.as_bytes()).unwrap();
    f
}

fn vectors() -> Value {
    serde_json::from_str(PACKAGES).unwrap()
}

fn package(name: &str) -> NamedTempFile {
    let doc = vectors();
    let p = doc["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == name)
        .unwrap();
    temp(p["package"].as_str().unwrap())
}

fn manifest_key() -> NamedTempFile {
    temp(
        vectors()["manifest_key"]["public_key_pem"]
            .as_str()
            .unwrap(),
    )
}

fn udm() -> Command {
    Command::cargo_bin("udm").unwrap()
}

#[test]
fn verifies_a_package_with_its_pinned_key() {
    let pkg = package("valid");
    let key = manifest_key();
    udm()
        .args(["verify", "--summary", "--trust-key"])
        .arg(key.path())
        .arg(pkg.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""verdict":"verified""#));
}

#[test]
fn an_unpinned_package_is_intact_but_unconfirmed() {
    let pkg = package("valid");
    udm()
        .arg("verify")
        .arg(pkg.path())
        .assert()
        .code(3)
        .stdout(predicate::str::contains(r#""verdict":"intact_unpinned""#))
        .stderr(predicate::str::contains("--trust-key"));
}

#[test]
fn a_tampered_package_fails() {
    let key = manifest_key();
    for name in [
        "stored_event_edited",
        "received_event_edited",
        "manifest_edited",
    ] {
        let pkg = package(name);
        udm()
            .args(["verify", "--trust-key"])
            .arg(key.path())
            .arg(pkg.path())
            .assert()
            .code(1)
            .stdout(predicate::str::contains(r#""verdict":"failed""#));
    }
}

#[test]
fn verifies_an_hmac_batch_stream_from_stdin() {
    let batch: Value = serde_json::from_str(BATCH).unwrap();
    let events = batch["batch"]["events"].as_array().unwrap();
    let mut ndjson = String::new();
    for e in events {
        ndjson.push_str(&serde_json::to_string(e).unwrap());
        ndjson.push('\n');
    }
    let key_id = events[0]["provenance"]["key_id"].as_str().unwrap();
    udm()
        .args(["verify", "-", "--hmac-key-id", key_id])
        .env("UDM_HMAC_SECRET", batch["batch"]["key"].as_str().unwrap())
        .write_stdin(ndjson.clone())
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""verified":3"#));

    // Without the secret's key the events cannot be checked, which is not
    // a failure.
    udm()
        .args(["verify", "-", "--summary"])
        .write_stdin(ndjson)
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""unknown_key":3"#));
}

#[test]
fn a_forged_event_fails() {
    let batch: Value = serde_json::from_str(BATCH).unwrap();
    let mut event = batch["batch"]["events"][0].clone();
    event["source_id"] = "someone-else".into();
    let f = temp(&serde_json::to_string_pretty(&event).unwrap());
    let key_id = event["provenance"]["key_id"].as_str().unwrap();
    udm()
        .args(["verify", "--hmac-key-id", key_id])
        .arg(f.path())
        .env("UDM_HMAC_SECRET", batch["batch"]["key"].as_str().unwrap())
        .assert()
        .code(1)
        .stdout(predicate::str::contains("invalid_signature"));
}
