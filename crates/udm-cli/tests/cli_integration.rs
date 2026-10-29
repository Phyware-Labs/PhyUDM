//! End-to-end CLI integration tests using `assert_cmd`.

use std::io::Write;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::NamedTempFile;

const FIXTURE: &str = include_str!("../../udm-eventstore/test-data/conformance.ndjson");

fn ndjson_tempfile() -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("temp");
    f.write_all(FIXTURE.as_bytes()).expect("write");
    f
}

fn store_arg(path: &std::path::Path) -> String {
    format!("memory://{}", path.display())
}

#[test]
fn shows_help() {
    Command::cargo_bin("udm")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("validate"))
        .stdout(predicate::str::contains("query"))
        .stdout(predicate::str::contains("timeline"));
}

#[test]
fn validates_a_canonical_event() {
    let mut f = NamedTempFile::new().unwrap();
    let event = r#"{
        "udm_version": "0.0.3",
        "event_id": "test-1",
        "event_type": "telemetry_periodic",
        "source_id": "amr-001",
        "source_type": "amr",
        "captured_at": "2026-06-07T19:00:00Z"
    }"#;
    f.write_all(event.as_bytes()).unwrap();

    Command::cargo_bin("udm")
        .unwrap()
        .args(["validate", f.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""ok":true"#));
}

#[test]
fn rejects_invalid_event_type() {
    let mut f = NamedTempFile::new().unwrap();
    let event = r#"{
        "udm_version": "0.0.3",
        "event_id": "test-1",
        "event_type": "totally_invalid",
        "source_id": "amr-001",
        "source_type": "amr",
        "captured_at": "2026-06-07T19:00:00Z"
    }"#;
    f.write_all(event.as_bytes()).unwrap();

    Command::cargo_bin("udm")
        .unwrap()
        .args(["validate", f.path().to_str().unwrap()])
        .assert()
        .failure()
        .stdout(predicate::str::contains(r#""ok":false"#));
}

#[test]
fn schema_show_dumps_event_schema() {
    Command::cargo_bin("udm")
        .unwrap()
        .args(["schema", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""$id""#))
        .stdout(predicate::str::contains("UDM Event"));
}

#[test]
fn explain_describes_safety_state_enum() {
    Command::cargo_bin("udm")
        .unwrap()
        .args(["explain", "safety/safety_state"])
        .assert()
        .success()
        .stdout(predicate::str::contains("emergency_stop"))
        .stdout(predicate::str::contains("normal"));
}

#[test]
fn template_produces_valid_skeleton() {
    Command::cargo_bin("udm")
        .unwrap()
        .args([
            "template",
            "--source-type",
            "amr",
            "--event-type",
            "telemetry_periodic",
            "--domains",
            "identity,location",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""identity""#))
        .stdout(predicate::str::contains(r#""location""#));
}

#[test]
fn query_against_memory_adapter() {
    let f = ndjson_tempfile();
    Command::cargo_bin("udm")
        .unwrap()
        .args(["--store", &store_arg(f.path()), "query"])
        .assert()
        .success()
        .stdout(predicate::str::contains("amr-001"))
        .stdout(predicate::str::contains("agv-002"));
}

#[test]
fn query_with_filter_narrows_results() {
    let f = ndjson_tempfile();
    let assert = Command::cargo_bin("udm")
        .unwrap()
        .args([
            "--store",
            &store_arg(f.path()),
            "query",
            "--filter",
            "event_type=safety_violation",
        ])
        .assert()
        .success();
    let stdout_bytes = assert.get_output().stdout.clone();
    let stdout = String::from_utf8(stdout_bytes).unwrap();
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(
        lines.len(),
        1,
        "expected one safety_violation, got: {stdout}"
    );
    assert!(lines[0].contains("safety_violation"));
}

#[test]
fn timeline_for_one_source_orders_ascending() {
    let f = ndjson_tempfile();
    Command::cargo_bin("udm")
        .unwrap()
        .args([
            "--store",
            &store_arg(f.path()),
            "timeline",
            "amr-001",
            "--from",
            "2026-06-07T00:00:00Z",
            "--to",
            "2026-06-08T00:00:00Z",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("amr-001"))
        .stdout(predicate::str::contains("19:00:00Z"));
}

#[test]
fn aggregate_count_by_source_id() {
    let f = ndjson_tempfile();
    Command::cargo_bin("udm")
        .unwrap()
        .args([
            "--store",
            &store_arg(f.path()),
            "aggregate",
            "--by",
            "source_id",
            "--agg",
            "count",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""amr-001""#))
        .stdout(predicate::str::contains(r#""event_count":3"#));
}

#[test]
fn audit_iso_ts_15066() {
    let f = ndjson_tempfile();
    Command::cargo_bin("udm")
        .unwrap()
        .args([
            "--store",
            &store_arg(f.path()),
            "audit",
            "iso-ts-15066",
            "--from",
            "2026-06-07T00:00:00Z",
            "--to",
            "2026-06-08T00:00:00Z",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("iso-ts-15066"))
        .stdout(predicate::str::contains(r#""matching_event_count""#));
}

#[test]
fn correlate_around_safety_event() {
    let f = ndjson_tempfile();
    Command::cargo_bin("udm")
        .unwrap()
        .args([
            "--store",
            &store_arg(f.path()),
            "correlate",
            "01940000-0000-7000-8000-000000000005",
            "--window",
            "30s",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""seed_event_id""#))
        .stdout(predicate::str::contains(r#""window_secs":30"#));
}

#[test]
fn analysis_without_store_errors_with_usage() {
    Command::cargo_bin("udm")
        .unwrap()
        .args(["query"])
        .env_remove("UDM_STORE")
        .assert()
        .failure()
        .stderr(predicate::str::contains("UDM_STORE"));
}

#[test]
fn phycloud_unreachable_endpoint_fails_cleanly() {
    // The PhyCloud adapter implements the read path; against an endpoint that
    // cannot be reached it must fail with a clear backend error, not panic.
    Command::cargo_bin("udm")
        .unwrap()
        .args(["--store", "phycloud://phycloud.invalid", "query"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("backend unavailable"));
}
