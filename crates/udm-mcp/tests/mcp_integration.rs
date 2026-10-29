//! End-to-end test for `udm-mcp`: spawn the binary, send JSON-RPC,
//! verify tool calls return expected data.

use std::io::Write;
use std::process::{Command, Stdio};

const FIXTURE: &str = include_str!("../../udm-eventstore/test-data/conformance.ndjson");

fn rpc_session() -> String {
    [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"0.0.1"}}}"#,
        r#"{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"list_event_types","arguments":{"filter":"safety"}}}"#,
        r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"query_events","arguments":{"filters":["event_type=safety_violation"]}}}"#,
        r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"validate_udm_event","arguments":{"payload":{"udm_version":"0.0.3","event_id":"e1","event_type":"telemetry_periodic","source_id":"amr-001","source_type":"amr","captured_at":"2026-06-07T19:00:00Z"}}}}"#,
        r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"validate_udm_event","arguments":{"payload":{"udm_version":"0.0.3","event_id":"e2","event_type":"NOT_A_REAL_TYPE","source_id":"amr-001","source_type":"amr","captured_at":"2026-06-07T19:00:00Z"}}}}"#,
        r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"aggregate","arguments":{"agg_fn":"count","group_by":["source_id"]}}}"#,
    ]
    .join("\n")
}

#[test]
fn mcp_server_serves_tools_over_stdio() {
    let mut fixture = tempfile::NamedTempFile::new().unwrap();
    fixture.write_all(FIXTURE.as_bytes()).unwrap();

    let bin = env!("CARGO_BIN_EXE_udm-mcp");

    let mut child = Command::new(bin)
        .arg("--store")
        .arg(format!("memory://{}", fixture.path().display()))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn udm-mcp");

    let session = rpc_session();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(session.as_bytes())
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(b"\n").unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("wait_with_output");
    let stdout = String::from_utf8(output.stdout).unwrap();

    // tools/list should mention every tool name we registered.
    for tool in [
        "query_events",
        "get_event",
        "timeline",
        "correlate_events",
        "aggregate",
        "compliance_audit",
        "incident_reconstruction",
        "explain_field",
        "list_event_types",
        "validate_udm_event",
    ] {
        assert!(
            stdout.contains(&format!("\"name\":\"{tool}\"")),
            "tools/list missing {tool}: {stdout}"
        );
    }

    // list_event_types filter=safety should return safety_violation.
    assert!(stdout.contains("safety_violation"));

    // query_events filter=event_type=safety_violation returns at least one event.
    assert!(stdout.contains("\\\"event_type\\\": \\\"safety_violation\\\""));

    // validate_udm_event on a good payload returns ok:true.
    assert!(stdout.contains("\\\"ok\\\": true"));

    // validate_udm_event on a bad payload returns ok:false.
    assert!(stdout.contains("\\\"ok\\\": false"));

    // aggregate count by source_id returns the amr-001 bucket
    // (3 events under that source in the fixture).
    assert!(stdout.contains("\\\"event_count\\\": 3"));
}
