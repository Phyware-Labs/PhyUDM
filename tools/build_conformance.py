#!/usr/bin/env python3
"""Generate the PhyUDM conformance fixture suite.

Produces ~100 fixtures partitioned across ``conformance/valid``,
``conformance/invalid``, ``conformance/edge`` and ``conformance/legacy``,
each paired with a ``meta.yaml`` describing intent and expected outcome.

Run from the repo root::

    python3 tools/build_conformance.py

The output JSON / YAML files are the canonical artifacts; this script is
the maintenance tool. Hand-edits to generated files will be overwritten —
add new templates to this generator instead.
"""

from __future__ import annotations

import json
import shutil
from copy import deepcopy
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parent.parent
CONFORMANCE_DIR = REPO_ROOT / "conformance"
SCHEMA_VERSION = "0.0.3"

EVENT_SCHEMA = "event.schema.json"
DOMAIN_SCHEMA_TEMPLATE = "domains/{slug}.schema.json"

# ---------------------------------------------------------------------------
# Canonical "good" envelope used as a seed for valid + edge fixtures.
# ---------------------------------------------------------------------------

BASE_ENVELOPE: dict[str, Any] = {
    "udm_version": SCHEMA_VERSION,
    "event_id": "01940000-0000-7000-8000-000000000001",
    "event_type": "telemetry_periodic",
    "source_id": "amr-001",
    "source_type": "amr",
    "captured_at": "2026-06-07T19:00:00Z",
}

# Domain definitions matching tools/build_schemas.py — kept in sync by hand.
DOMAINS: list[tuple[int, str, dict[str, Any]]] = [
    (1, "identity", {
        "source_id": "amr-001",
        "source_type": "amr",
        "platform": "MiR250",
        "model": "MiR250-rev3",
        "serial_number": "SN-1234567",
        "firmware_version": "2.18.4",
        "fleet_id": "fleet-warehouse-a",
        "site_id": "site-dallas-01",
        "organization_id": "org-acme",
        "display_name": "Forklift Charlie",
        "tags": ["picking", "zone-a"],
        "metadata": {"team": "ops", "region": "us-tx"},
        "mac_address": "AA:BB:CC:DD:EE:FF",
        "ip_address": "10.0.1.42",
    }),
    (2, "location", {
        "latitude": 32.7767,
        "longitude": -96.7970,
        "altitude_m": 134.5,
        "heading_deg": 270.0,
        "local": {"x_m": 12.5, "y_m": 4.25, "z_m": 0.0},
        "horizontal_accuracy_m": 0.05,
        "vertical_accuracy_m": 0.5,
        "frame_id": "map",
        "map_id": "warehouse-a-floor-1",
        "floor": 1,
    }),
    (3, "motion", {
        "speed_mps": 1.2,
        "motion_state": "moving",
        "frame_id": "base_link",
        "commanded_linear_mps": 1.2,
        "commanded_angular_dps": 0.0,
    }),
    (4, "power", {
        "battery": {"soc_pct": 78.4, "voltage_v": 48.2},
        "charging": {"is_charging": False},
        "power_consumption_w": 145.2,
        "average_power_w": 132.0,
        "power_source": "battery",
        "voltage_v": 48.2,
        "current_a": 3.01,
        "estimated_runtime_min": 240.0,
        "estimated_range_m": 14400.0,
    }),
    (5, "operational", {
        "mode": "autonomous",
        "state": "executing_task",
        "previous_state": "ready",
        "time_in_state_sec": 124.5,
        "task": {"id": "task-987", "kind": "pick_and_place"},
        "error_count": 0,
        "is_available": True,
        "uptime_sec": 36000.0,
        "mission_id": "mission-2026-06-07-001",
        "session_id": "sess-abc",
    }),
    (6, "navigation", {
        "localization": {"quality": "good"},
        "obstacle_count": 2,
        "obstacles": [
            {"kind": "human", "distance_m": 4.2},
            {"kind": "static", "distance_m": 1.8},
        ],
    }),
    (7, "perception", {
        "lidar": [{"status": "ok", "hz": 10}],
        "cameras": [{"status": "ok", "resolution": "1920x1080"}],
        "detection_count": 3,
    }),
    (8, "safety", {
        "safety_state": "normal",
        "is_safe": True,
        "protective_stop_active": False,
        "current_zone_ids": ["zone-a", "zone-warehouse"],
        "violation_count": 0,
        "speed_limit_mps": 1.5,
        "safety_system_ok": True,
        "last_safety_check": "2026-06-07T18:59:30Z",
    }),
    (9, "actuators", {
        "drive_motors": [{"id": "drive-l", "status": "ok"}, {"id": "drive-r", "status": "ok"}],
        "grippers": [{"id": "grip-1", "state": "open"}],
    }),
    (10, "communication", {
        "network": {"type": "wifi", "rssi_dbm": -52},
        "fleet": {"connected": True, "endpoint": "fleet.example.com"},
    }),
    (11, "compute", {
        "cpu": {"usage_pct": 32.4},
        "memory": {"used_mb": 2048, "total_mb": 8192},
        "uptime_sec": 36000.0,
        "load_average": [0.42, 0.51, 0.38],
        "kernel_version": "6.1.0-amd64",
        "os_version": "Ubuntu 22.04.4 LTS",
    }),
    (12, "ai", {
        "models": [{"id": "detector-v3", "type": "object_detection"}],
        "decisions": [{"id": "dec-1", "model_id": "detector-v3", "confidence": 0.92}],
        "total_inferences": 1543,
        "avg_inference_ms": 12.4,
    }),
    (13, "maintenance", {
        "health_score": 0.94,
        "operating_hours": 1248.5,
        "urgency": "routine",
    }),
    (14, "context", {
        "time": {"timezone": "America/Chicago"},
        "facility": {"id": "warehouse-a", "type": "warehouse"},
    }),
    (15, "payload", {
        "load_status": "partial_load",
        "total_weight_kg": 42.5,
        "max_weight_kg": 250.0,
        "weight_utilization_pct": 17.0,
        "item_count": 3,
        "is_secured": True,
        "center_of_mass_offset_m": [0.02, -0.01, 0.4],
    }),
    (16, "manipulation", {
        "arm_state": "moving",
        "control_mode": "position",
        "collision_avoidance_active": True,
    }),
    (17, "hri", {
        "interaction_state": "human_detected",
        "human_count": 1,
    }),
    (18, "coordination", {
        "fleet_role": "independent",
    }),
    (19, "simulation", {
        "is_simulated": False,
    }),
    (20, "thermal", {
        "thermal_state": "normal",
        "ambient_temp_c": 24.5,
        "is_throttling": False,
    }),
    (21, "audio", {
        "ambient_noise_db": 56.3,
    }),
    (22, "environment_interaction", {
        "doors": [{"id": "door-1", "state": "closed"}],
    }),
    (23, "compliance", {
        "certifications": [{"name": "ISO-13849", "status": "certified"}],
    }),
]


def _slug(num: int, key: str) -> str:
    """Return the schema-path slug used in DOMAIN_SCHEMA_TEMPLATE."""
    dash = key.replace("_", "-")
    return f"{num:02d}-{dash}"


def _write_pair(
    folder: Path,
    name: str,
    payload: dict[str, Any],
    meta: dict[str, Any],
) -> None:
    folder.mkdir(parents=True, exist_ok=True)
    json_path = folder / f"{name}.json"
    meta_path = folder / f"{name}.meta.yaml"
    with json_path.open("w", encoding="utf-8") as fh:
        json.dump(payload, fh, indent=2)
        fh.write("\n")
    with meta_path.open("w", encoding="utf-8") as fh:
        _dump_yaml(meta, fh)


def _dump_yaml(obj: Any, fh: Any, indent: int = 0) -> None:
    """Minimal YAML emitter — keeps the toolchain dep-free for fixture authoring."""
    pad = "  " * indent
    if isinstance(obj, dict):
        for k, v in obj.items():
            if isinstance(v, (dict, list)) and v:
                fh.write(f"{pad}{k}:\n")
                _dump_yaml(v, fh, indent + 1)
            else:
                fh.write(f"{pad}{k}: {_yaml_scalar(v)}\n")
    elif isinstance(obj, list):
        for item in obj:
            if isinstance(item, (dict, list)):
                fh.write(f"{pad}-\n")
                _dump_yaml(item, fh, indent + 1)
            else:
                fh.write(f"{pad}- {_yaml_scalar(item)}\n")
    else:
        fh.write(f"{pad}{_yaml_scalar(obj)}\n")


def _yaml_scalar(v: Any) -> str:
    if v is None:
        return "null"
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, (int, float)):
        return str(v)
    s = str(v)
    if any(c in s for c in ":#[]{},&*!|>'\"%@`\n") or s != s.strip():
        return json.dumps(s)
    return s


# ---------------------------------------------------------------------------
# Generators.
# ---------------------------------------------------------------------------

def _bump_event_id(seed: dict[str, Any], n: int) -> dict[str, Any]:
    out = deepcopy(seed)
    out["event_id"] = f"01940000-0000-7000-8000-{n:012d}"
    return out


def emit_valid_minimal() -> int:
    """One minimal envelope per event_type variant + one per source_type."""
    folder = CONFORMANCE_DIR / "valid"
    count = 0
    event_types = [
        "telemetry_periodic", "telemetry_on_change", "telemetry_snapshot",
        "state_transition", "mode_change",
        "task_started", "task_completed", "task_failed", "task_cancelled",
        "goal_reached", "path_blocked", "rerouting",
        "safety_violation", "emergency_stop",
        "system_startup", "system_shutdown",
        "error", "custom",
        # Added after the initial release; last so existing fixtures keep
        # their event ids.
        "system_heartbeat",
    ]
    for et in event_types:
        count += 1
        payload = _bump_event_id(BASE_ENVELOPE, count)
        payload["event_type"] = et
        _write_pair(folder, f"envelope-event-type-{et}", payload, {
            "purpose": f"Minimal envelope with event_type={et!r}.",
            "expected_result": "valid",
            "schema_path": EVENT_SCHEMA,
            "target_field_path": "event_type",
        })
    return count


def emit_valid_source_types() -> int:
    folder = CONFORMANCE_DIR / "valid"
    source_types = [
        "amr", "agv", "autonomous_forklift", "delivery_robot",
        "cobot", "industrial_arm", "mobile_manipulator",
        "autonomous_vehicle", "drone", "humanoid", "quadruped",
        "simulation",
    ]
    count = 0
    for n, st in enumerate(source_types, start=1):
        count += 1
        payload = _bump_event_id(BASE_ENVELOPE, 100 + n)
        payload["source_type"] = st
        payload["source_id"] = f"{st}-001"
        _write_pair(folder, f"envelope-source-type-{st}", payload, {
            "purpose": f"Minimal envelope with source_type={st!r}.",
            "expected_result": "valid",
            "schema_path": EVENT_SCHEMA,
            "target_field_path": "source_type",
        })
    return count


def emit_valid_per_domain() -> int:
    """One fixture per domain — envelope + that domain populated."""
    folder = CONFORMANCE_DIR / "valid"
    count = 0
    for n, key, sample in DOMAINS:
        count += 1
        payload = _bump_event_id(BASE_ENVELOPE, 200 + n)
        payload[key] = sample
        slug = _slug(n, key)
        _write_pair(folder, f"domain-{slug}", payload, {
            "purpose": f"Envelope + populated {key} domain.",
            "expected_result": "valid",
            "schema_path": EVENT_SCHEMA,
            "target_field_path": key,
            "domain_schema": DOMAIN_SCHEMA_TEMPLATE.format(slug=slug),
        })
    return count


def emit_valid_cross_cutting() -> int:
    """Realistic events touching multiple domains."""
    folder = CONFORMANCE_DIR / "valid"
    scenarios: list[tuple[str, dict[str, Any], str]] = [
        ("warehouse-amr-picking", {
            **_bump_event_id(BASE_ENVELOPE, 301),
            "event_type": "task_started",
            "identity": deepcopy(DOMAINS[0][2]),
            "location": deepcopy(DOMAINS[1][2]),
            "motion": deepcopy(DOMAINS[2][2]),
            "power": deepcopy(DOMAINS[3][2]),
            "operational": deepcopy(DOMAINS[4][2]),
            "navigation": deepcopy(DOMAINS[5][2]),
            "safety": deepcopy(DOMAINS[7][2]),
            "payload": deepcopy(DOMAINS[14][2]),
        }, "AMR starting a pick-and-place task with rich envelope."),
        ("cobot-grasp-success", {
            **_bump_event_id(BASE_ENVELOPE, 302),
            "event_type": "task_completed",
            "source_type": "cobot",
            "source_id": "cobot-005",
            "identity": {"source_id": "cobot-005", "source_type": "cobot",
                         "platform": "UR10e", "fleet_id": "cell-2"},
            "manipulation": {"arm_state": "at_target", "control_mode": "impedance",
                             "collision_avoidance_active": True,
                             "grasp": {"object_id": "widget-42", "phase": "secured"}},
            "actuators": deepcopy(DOMAINS[8][2]),
            "safety": {"safety_state": "normal", "is_safe": True,
                       "speed_limit_mps": 0.25, "safety_system_ok": True},
        }, "Cobot completing a successful grasp."),
        ("drone-mission-mode-change", {
            **_bump_event_id(BASE_ENVELOPE, 303),
            "event_type": "mode_change",
            "source_type": "drone",
            "source_id": "drone-aerial-12",
            "operational": {"mode": "autonomous", "state": "navigating",
                            "previous_state": "ready", "mission_id": "m-12-2026"},
            "location": {"latitude": 37.7749, "longitude": -122.4194,
                         "altitude_m": 80.5, "heading_deg": 180.0},
            "power": {"battery": {"soc_pct": 64.0}, "power_source": "battery"},
        }, "Drone transitioning to autonomous flight mode."),
        ("simulation-fleet-coordination", {
            **_bump_event_id(BASE_ENVELOPE, 304),
            "event_type": "telemetry_snapshot",
            "source_type": "simulation",
            "source_id": "sim-warehouse-001",
            "simulation": {"is_simulated": True,
                           "simulator": {"type": "gazebo", "version": "11.13"},
                           "time_scale": 1.0, "step_count": 482},
            "coordination": {"fleet_role": "coordinator",
                             "formation": {"type": "line", "members": 4}},
            "identity": {"source_id": "sim-warehouse-001",
                         "source_type": "simulation"},
        }, "Simulated fleet coordination snapshot."),
        ("safety-emergency-stop", {
            **_bump_event_id(BASE_ENVELOPE, 305),
            "event_type": "emergency_stop",
            "safety": {"safety_state": "emergency_stop", "is_safe": False,
                       "e_stop": {"type": "hardware", "triggered_at": "2026-06-07T19:00:01Z"},
                       "protective_stop_active": True,
                       "protective_stop_reason": "operator_button",
                       "violation_count": 1,
                       "violations": [{"type": "proximity_violation",
                                       "severity": "high"}]},
            "operational": {"mode": "emergency", "state": "emergency_stopped"},
        }, "Hardware e-stop triggered with safety + operational context."),
        ("hri-handover-completed", {
            **_bump_event_id(BASE_ENVELOPE, 306),
            "event_type": "task_completed",
            "source_type": "humanoid",
            "source_id": "humanoid-care-3",
            "hri": {"interaction_state": "handover",
                    "human_count": 1,
                    "handover": {"state": "complete", "object_id": "cup-7"}},
            "manipulation": {"arm_state": "home",
                             "end_effector": {"id": "ee-soft-grip"}},
        }, "Humanoid completing a handover with HRI + manipulation context."),
        ("compliance-iso-attestation", {
            **_bump_event_id(BASE_ENVELOPE, 307),
            "event_type": "telemetry_periodic",
            "compliance": {"certifications": [
                {"name": "ISO-13849", "status": "certified",
                 "valid_until": "2027-01-01"},
                {"name": "ISO-3691-4", "status": "certified"},
            ], "functional_safety": {"performance_level": "pl_d"}},
        }, "Periodic compliance attestation snapshot."),
        ("ai-decision-trace", {
            **_bump_event_id(BASE_ENVELOPE, 308),
            "event_type": "telemetry_on_change",
            "ai": deepcopy(DOMAINS[11][2]),
            "perception": {"detection_count": 5,
                           "detections": [{"id": "d1", "label": "person",
                                           "confidence": 0.93}]},
        }, "AI decision trace with paired perception detection."),
        ("thermal-throttling-event", {
            **_bump_event_id(BASE_ENVELOPE, 309),
            "event_type": "state_transition",
            "thermal": {"thermal_state": "high",
                        "ambient_temp_c": 42.1,
                        "is_throttling": True},
            "compute": {"cpu": {"usage_pct": 92.0, "temp_c": 88.0}},
        }, "Compute thermal throttling event."),
        ("maintenance-urgent-event", {
            **_bump_event_id(BASE_ENVELOPE, 310),
            "event_type": "error",
            "maintenance": {"health_score": 0.42,
                            "urgency": "urgent",
                            "operating_hours": 14250.0,
                            "components": [{"id": "drive-l",
                                            "health": "needs_attention"}]},
        }, "Maintenance health degradation requiring attention."),
        ("audio-alarm-detection", {
            **_bump_event_id(BASE_ENVELOPE, 311),
            "event_type": "telemetry_on_change",
            "audio": {"ambient_noise_db": 84.2,
                      "sound_detection": [{"type": "alarm", "confidence": 0.97}]},
        }, "Alarm sound detected via audio domain."),
        ("environment-elevator-boarding", {
            **_bump_event_id(BASE_ENVELOPE, 312),
            "event_type": "telemetry_on_change",
            "environment_interaction": {"elevators": [
                {"id": "elev-1", "state": "boarding", "target_floor": 3}
            ]},
            "location": {"floor": 1, "frame_id": "map"},
        }, "Robot boarding an elevator."),
    ]
    for name, payload, purpose in scenarios:
        _write_pair(folder, name, payload, {
            "purpose": purpose,
            "expected_result": "valid",
            "schema_path": EVENT_SCHEMA,
        })
    return len(scenarios)


def emit_valid_extensions() -> int:
    folder = CONFORMANCE_DIR / "valid"
    cases = [
        ("extensions-vendor-namespace", {
            **_bump_event_id(BASE_ENVELOPE, 401),
            "extensions": {
                "acme_vendor": {"firmware_build": "abc123",
                                 "feature_flags": ["x", "y"]},
                "custom_metric_a": 0.123,
            },
        }, "Envelope + extensions with vendor-namespaced keys."),
        ("provenance-signed-event", {
            **_bump_event_id(BASE_ENVELOPE, 402),
            "provenance": {
                "signature": "deadbeef" * 8,
                "key_id": "key-2026-q2",
                "algorithm": "hmac-sha256",
                "signed_fields": ["event_id", "captured_at", "source_id"],
                "signed_at": "2026-06-07T19:00:00Z",
            },
        }, "Envelope + provenance block (signed event)."),
    ]
    for name, payload, purpose in cases:
        _write_pair(folder, name, payload, {
            "purpose": purpose,
            "expected_result": "valid",
            "schema_path": EVENT_SCHEMA,
        })
    return len(cases)


# Provenance and heartbeat (spec/provenance.md).
ES256_PROVENANCE: dict[str, Any] = {
    "signer": "tpm",
    "algorithm": "es256",
    "key_id": "5f1c" * 16,
    "signed_at": "2026-06-07T19:00:00.250000+00:00",
    "sequence_num": 42,
    "boot_epoch": "0123456789abcdef0123456789abcdef",
    "boot_counter": 7,
    "agent_id": "agent-amr-001",
    "device_attestation": {
        "mode": "tpm",
        "quote_id": "q-2026-06-07-001",
        "quote_status": "verified",
    },
    "signature": "ab" * 64,
}

BATCH: dict[str, Any] = {
    "statement": {
        "type": "udm.merkle-batch.v1",
        "batch_id": "b0000000000000000000000000000001",
        "root": "31" * 32,
        "leaf_count": 3,
        "first_sequence": 41,
        "last_sequence": 43,
        "boot_epoch": "0123456789abcdef0123456789abcdef",
        "key_id": "5f1c" * 16,
        "signed_at": "2026-06-07T19:00:03.500000+00:00",
    },
    "signature": "cd" * 64,
    "leaf_index": 1,
    "proof": ["0a" * 32, "0b" * 32],
}


def emit_valid_provenance() -> int:
    folder = CONFORMANCE_DIR / "valid"
    batch_provenance = {
        k: v for k, v in ES256_PROVENANCE.items() if k != "signature"
    }
    batch_provenance["sequence_num"] = 42
    batch_provenance["batch"] = deepcopy(BATCH)
    cases = [
        ("provenance-es256-signed", {
            **_bump_event_id(BASE_ENVELOPE, 410),
            "provenance": deepcopy(ES256_PROVENANCE),
        }, "Event signed with a hardware key (es256), with sequence, boot counter "
           "and TPM attestation."),
        ("provenance-hmac-development", {
            **_bump_event_id(BASE_ENVELOPE, 411),
            "provenance": {
                "signer": "hmac",
                "algorithm": "hmac-sha256",
                "key_id": "dev-key-1",
                "signed_at": "2026-06-07T19:00:00Z",
                "sequence_num": 1,
                "boot_epoch": "fedcba9876543210fedcba9876543210",
                "device_attestation": {
                    "mode": "software",
                    "warning": "no hardware root of trust",
                },
                "signature": "3183f961" * 8,
            },
        }, "Development event signed with a shared secret, without a hardware root of trust."),
        ("provenance-batch-signed", {
            **_bump_event_id(BASE_ENVELOPE, 412),
            "provenance": batch_provenance,
        }, "Event covered by a Merkle batch signature instead of its own."),
        ("heartbeat-event", {
            **_bump_event_id(BASE_ENVELOPE, 413),
            "event_type": "system_heartbeat",
            "heartbeat": {
                "interval_sec": 30,
                "agent_id": "agent-amr-001",
                "agent_version": "0.3.1",
            },
            "provenance": deepcopy(ES256_PROVENANCE),
        }, "Signed liveness heartbeat."),
    ]
    for name, payload, purpose in cases:
        _write_pair(folder, name, payload, {
            "purpose": purpose,
            "expected_result": "valid",
            "schema_path": EVENT_SCHEMA,
        })
    return len(cases)


def emit_invalid_provenance() -> int:
    folder = CONFORMANCE_DIR / "invalid"
    cases: list[tuple[str, dict[str, Any], str, str, str]] = []

    def with_provenance(n: int, **changes: Any) -> dict[str, Any]:
        out = _bump_event_id(BASE_ENVELOPE, n)
        out["provenance"] = deepcopy(ES256_PROVENANCE)
        out["provenance"].update(changes)
        return out

    cases.append(("provenance-signature-not-hex",
                  with_provenance(600, signature="ed25519:abc"),
                  "Signatures are lowercase hex.",
                  "provenance.signature", "pattern"))
    cases.append(("provenance-sequence-num-zero",
                  with_provenance(601, sequence_num=0),
                  "Provenance sequence numbers start at 1.",
                  "provenance.sequence_num", "minimum"))
    cases.append(("provenance-boot-counter-negative",
                  with_provenance(602, boot_counter=-1),
                  "Boot counters never go below 0.",
                  "provenance.boot_counter", "minimum"))
    cases.append(("provenance-attestation-unknown-mode",
                  with_provenance(603, device_attestation={"mode": "trusted"}),
                  "Attestation mode is 'tpm' or 'software'.",
                  "provenance.device_attestation.mode", "enum"))

    batch = deepcopy(BATCH)
    del batch["proof"]
    cases.append(("provenance-batch-missing-proof",
                  with_provenance(604, batch=batch),
                  "A batch signature carries the leaf's audit path.",
                  "provenance.batch.proof", "required"))
    batch = deepcopy(BATCH)
    batch["statement"]["root"] = "not-a-hash"
    cases.append(("provenance-batch-bad-root",
                  with_provenance(605, batch=batch),
                  "A batch root is a lowercase hex SHA-256.",
                  "provenance.batch.statement.root", "pattern"))

    for n, heartbeat, purpose, target, keyword in [
        (606, {"interval_sec": 0}, "Heartbeat intervals are at least 1 second.",
         "heartbeat.interval_sec", "minimum"),
        (607, {"agent_id": "agent-amr-001"}, "A heartbeat states its interval.",
         "heartbeat.interval_sec", "required"),
    ]:
        payload = _bump_event_id(BASE_ENVELOPE, n)
        payload["event_type"] = "system_heartbeat"
        payload["heartbeat"] = heartbeat
        name = "heartbeat-interval-zero" if n == 606 else "heartbeat-missing-interval"
        cases.append((name, payload, purpose, target, keyword))

    for name, payload, purpose, target, keyword in cases:
        _write_pair(folder, name, payload, {
            "purpose": purpose,
            "expected_result": "invalid",
            "schema_path": EVENT_SCHEMA,
            "target_field_path": target,
            "expected_failure_keyword": keyword,
        })
    return len(cases)


def emit_invalid_envelope() -> int:
    folder = CONFORMANCE_DIR / "invalid"
    cases: list[tuple[str, dict[str, Any], str, str, str]] = []

    # Missing required fields.
    for missing in ["udm_version", "event_id", "event_type",
                    "source_id", "source_type", "captured_at"]:
        payload = _bump_event_id(BASE_ENVELOPE, 500 + len(cases))
        del payload[missing]
        cases.append((
            f"envelope-missing-{missing}",
            payload,
            f"Required envelope field {missing!r} omitted.",
            missing, "required",
        ))

    # Bad enum values.
    bad_enum_cases = [
        ("envelope-bad-event-type", "event_type", "TelemetryPeriodic"),
        ("envelope-bad-event-type-spec-form", "event_type", "telemetry.periodic"),
        ("envelope-bad-source-type", "source_type", "MOBILE_ROBOT"),
    ]
    for name, field, value in bad_enum_cases:
        payload = _bump_event_id(BASE_ENVELOPE, 510 + len(cases))
        payload[field] = value
        cases.append((
            name, payload,
            f"Non-canonical enum value for {field}: {value!r}.",
            field, "enum",
        ))

    # Bad scalar types / formats.
    bad_scalar_cases = [
        ("envelope-bad-version-format", "udm_version", "1.0",
         "udm_version must be full SemVer (major.minor.patch).", "pattern"),
        ("envelope-bad-captured-at", "captured_at", "yesterday",
         "captured_at must be ISO-8601 date-time.", "format"),
        ("envelope-bad-sequence-num", "sequence_num", -5,
         "sequence_num must be a non-negative integer.", "minimum"),
        ("envelope-empty-event-id", "event_id", "",
         "event_id must be non-empty.", "minLength"),
        ("envelope-non-string-source-id", "source_id", 12345,
         "source_id must be a string.", "type"),
    ]
    for name, field, value, purpose, keyword in bad_scalar_cases:
        payload = _bump_event_id(BASE_ENVELOPE, 520 + len(cases))
        payload[field] = value
        cases.append((name, payload, purpose, field, keyword))

    for name, payload, purpose, target, keyword in cases:
        _write_pair(folder, name, payload, {
            "purpose": purpose,
            "expected_result": "invalid",
            "schema_path": EVENT_SCHEMA,
            "target_field_path": target,
            "expected_failure_keyword": keyword,
        })
    return len(cases)


def emit_invalid_domain() -> int:
    folder = CONFORMANCE_DIR / "invalid"
    cases: list[tuple[str, dict[str, Any], str, str, str]] = []
    n = 600

    def _mk(domain: str, body: dict[str, Any]) -> dict[str, Any]:
        nonlocal n; n += 1
        p = _bump_event_id(BASE_ENVELOPE, n)
        p[domain] = body
        return p

    cases += [
        ("identity-bad-mac", _mk("identity", {"mac_address": "not-a-mac"}),
         "identity.mac_address must match MAC pattern.",
         "identity/mac_address", "pattern"),
        ("identity-tags-non-array", _mk("identity", {"tags": "single-tag"}),
         "identity.tags must be an array.",
         "identity/tags", "type"),
        ("identity-metadata-non-string-value",
         _mk("identity", {"metadata": {"k": 42}}),
         "identity.metadata values must be strings.",
         "identity/metadata", "type"),
        ("location-bad-latitude", _mk("location", {"latitude": 200.0}),
         "location.latitude exceeds 90 degrees.",
         "location/latitude", "maximum"),
        ("location-bad-longitude", _mk("location", {"longitude": -999.0}),
         "location.longitude below -180 degrees.",
         "location/longitude", "minimum"),
        ("location-bad-heading", _mk("location", {"heading_deg": 720.0}),
         "location.heading_deg ≥ 360.",
         "location/heading_deg", "maximum"),
        ("location-negative-accuracy",
         _mk("location", {"horizontal_accuracy_m": -1.0}),
         "location.horizontal_accuracy_m must be non-negative.",
         "location/horizontal_accuracy_m", "minimum"),
        ("motion-bad-motion-state",
         _mk("motion", {"motion_state": "spinning"}),
         "motion.motion_state not in enum.",
         "motion/motion_state", "enum"),
        ("motion-negative-speed",
         _mk("motion", {"speed_mps": -2.0}),
         "motion.speed_mps must be non-negative.",
         "motion/speed_mps", "minimum"),
        ("power-bad-power-source",
         _mk("power", {"power_source": "diesel"}),
         "power.power_source not in enum.",
         "power/power_source", "enum"),
        ("power-negative-consumption",
         _mk("power", {"power_consumption_w": -10.0}),
         "power.power_consumption_w must be non-negative.",
         "power/power_consumption_w", "minimum"),
        ("operational-bad-mode",
         _mk("operational", {"mode": "totally_autonomous"}),
         "operational.mode not in enum.",
         "operational/mode", "enum"),
        ("operational-bad-state",
         _mk("operational", {"state": "in_progress"}),
         "operational.state not in enum.",
         "operational/state", "enum"),
        ("operational-negative-error-count",
         _mk("operational", {"error_count": -1}),
         "operational.error_count must be ≥ 0.",
         "operational/error_count", "minimum"),
        ("safety-bad-state",
         _mk("safety", {"safety_state": "panic"}),
         "safety.safety_state not in enum.",
         "safety/safety_state", "enum"),
        ("safety-current-zone-ids-non-array",
         _mk("safety", {"current_zone_ids": "zone-a"}),
         "safety.current_zone_ids must be array of strings.",
         "safety/current_zone_ids", "type"),
        ("payload-bad-load-status",
         _mk("payload", {"load_status": "half"}),
         "payload.load_status not in enum.",
         "payload/load_status", "enum"),
        ("payload-util-over-100",
         _mk("payload", {"weight_utilization_pct": 150.0}),
         "payload.weight_utilization_pct > 100.",
         "payload/weight_utilization_pct", "maximum"),
        ("payload-com-wrong-length",
         _mk("payload", {"center_of_mass_offset_m": [0.0, 0.0]}),
         "payload.center_of_mass_offset_m must have 3 entries.",
         "payload/center_of_mass_offset_m", "minItems"),
        ("manipulation-bad-arm-state",
         _mk("manipulation", {"arm_state": "wiggling"}),
         "manipulation.arm_state not in enum.",
         "manipulation/arm_state", "enum"),
        ("manipulation-bad-control-mode",
         _mk("manipulation", {"control_mode": "pid"}),
         "manipulation.control_mode not in enum.",
         "manipulation/control_mode", "enum"),
        ("hri-bad-interaction-state",
         _mk("hri", {"interaction_state": "chatting"}),
         "hri.interaction_state not in enum.",
         "hri/interaction_state", "enum"),
        ("hri-negative-human-count",
         _mk("hri", {"human_count": -1}),
         "hri.human_count must be ≥ 0.",
         "hri/human_count", "minimum"),
        ("coordination-bad-fleet-role",
         _mk("coordination", {"fleet_role": "boss"}),
         "coordination.fleet_role not in enum.",
         "coordination/fleet_role", "enum"),
        ("thermal-bad-state",
         _mk("thermal", {"thermal_state": "hot"}),
         "thermal.thermal_state not in enum.",
         "thermal/thermal_state", "enum"),
        ("maintenance-health-score-over-1",
         _mk("maintenance", {"health_score": 1.5}),
         "maintenance.health_score > 1.0.",
         "maintenance/health_score", "maximum"),
        ("maintenance-bad-urgency",
         _mk("maintenance", {"urgency": "later"}),
         "maintenance.urgency not in enum.",
         "maintenance/urgency", "enum"),
        ("compute-load-average-wrong-arity",
         _mk("compute", {"load_average": [0.4, 0.5]}),
         "compute.load_average must have exactly 3 entries.",
         "compute/load_average", "minItems"),
        ("simulation-negative-time-scale",
         _mk("simulation", {"time_scale": -1.0}),
         "simulation.time_scale must be non-negative.",
         "simulation/time_scale", "minimum"),
        ("simulation-bad-step-count-type",
         _mk("simulation", {"step_count": "many"}),
         "simulation.step_count must be integer.",
         "simulation/step_count", "type"),
        ("provenance-non-object",
         {**_bump_event_id(BASE_ENVELOPE, 700), "provenance": "signed"},
         "provenance must be an object, not a string.",
         "provenance", "type"),
        ("extensions-non-object",
         {**_bump_event_id(BASE_ENVELOPE, 701), "extensions": ["x", "y"]},
         "extensions must be an object, not an array.",
         "extensions", "type"),
    ]
    for name, payload, purpose, target, keyword in cases:
        _write_pair(folder, name, payload, {
            "purpose": purpose,
            "expected_result": "invalid",
            "schema_path": EVENT_SCHEMA,
            "target_field_path": target,
            "expected_failure_keyword": keyword,
        })
    return len(cases)


def emit_edge() -> int:
    folder = CONFORMANCE_DIR / "edge"
    cases: list[tuple[str, dict[str, Any], str]] = [
        ("edge-empty-domains", _bump_event_id(BASE_ENVELOPE, 800),
         "Envelope only; no domain payloads."),
        ("edge-location-boundary-coords", {
            **_bump_event_id(BASE_ENVELOPE, 801),
            "location": {"latitude": 90.0, "longitude": -180.0,
                         "heading_deg": 0.0, "altitude_m": 8848.86},
        }, "Boundary lat/lon/heading values (90, -180, 0)."),
        ("edge-location-south-pole", {
            **_bump_event_id(BASE_ENVELOPE, 802),
            "location": {"latitude": -90.0, "longitude": 180.0,
                         "heading_deg": 359.999},
        }, "Boundary lat/lon at south pole + dateline."),
        ("edge-zero-and-empty-collections", {
            **_bump_event_id(BASE_ENVELOPE, 803),
            "operational": {"errors": [], "error_count": 0},
            "navigation": {"obstacles": [], "obstacle_count": 0},
            "perception": {"detections": [], "detection_count": 0},
            "safety": {"current_zone_ids": [], "violation_count": 0,
                       "violations": []},
        }, "Empty arrays and zero counts across domains."),
        ("edge-unicode-strings", {
            **_bump_event_id(BASE_ENVELOPE, 804),
            "source_id": "ロボット-🤖-001",
            "identity": {"source_id": "ロボット-🤖-001",
                         "source_type": "amr",
                         "display_name": "Robôt Émile",
                         "tags": ["zëtà", "αβγ", "中文"]},
        }, "Unicode in source_id, display_name, and tags."),
        ("edge-large-numbers", {
            **_bump_event_id(BASE_ENVELOPE, 805),
            "operational": {"uptime_sec": 1.0e9},
            "compute": {"uptime_sec": 1.0e9, "load_average": [128.0, 128.0, 128.0]},
            "ai": {"total_inferences": 9_000_000_000},
            "simulation": {"step_count": 4_000_000_000},
        }, "Very large numeric values across domains."),
        ("edge-rich-extensions", {
            **_bump_event_id(BASE_ENVELOPE, 806),
            "extensions": {
                "deep": {"nested": {"more": {"and": {"more": True}}}},
                "list_of_lists": [[1, 2], [3, 4]],
                "vendor_x": {"build_sha": "deadbeef"},
            },
        }, "Deeply-nested extensions payload."),
        ("edge-all-23-domains-empty", {
            **_bump_event_id(BASE_ENVELOPE, 807),
            **{key: {} for _, key, _ in DOMAINS},
        }, "All 23 domain slots present but empty."),
        ("edge-optional-envelope-fields", {
            **_bump_event_id(BASE_ENVELOPE, 808),
            "received_at": "2026-06-07T19:00:05Z",
            "sent_at": "2026-06-07T19:00:04Z",
            "sequence_num": 0,
            "session_id": "sess-edge",
            "sdk_version": "rust-sdk-0.4.2",
        }, "All optional envelope fields populated."),
        ("edge-event-type-custom", {
            **_bump_event_id(BASE_ENVELOPE, 809),
            "event_type": "custom",
            "extensions": {"custom_event_subtype": "domain_specific_thing"},
        }, "event_type=custom with subtype in extensions."),
        ("edge-source-type-custom", {
            **_bump_event_id(BASE_ENVELOPE, 810),
            "source_type": "custom",
            "source_id": "custom-platform-001",
            "identity": {"source_id": "custom-platform-001",
                         "source_type": "custom",
                         "platform": "VendorY-Z9"},
        }, "source_type=custom for a vendor-specific platform."),
        ("edge-precise-timestamps", {
            **_bump_event_id(BASE_ENVELOPE, 811),
            "captured_at": "2026-06-07T19:00:00.123456789Z",
            "received_at": "2026-06-07T19:00:00.123456789+00:00",
        }, "Sub-second precision + explicit-offset timestamp formats."),
        ("edge-extensions-license-metadata", {
            **_bump_event_id(BASE_ENVELOPE, 812),
            "extensions": {
                "_license_status": "active",
                "_license_tenant_id": "tenant-acme",
                "_license_plan": "enterprise",
                "_quarantine": False,
            },
        }, "PhyTrace-style license metadata in extensions."),
        ("edge-provenance-and-extensions", {
            **_bump_event_id(BASE_ENVELOPE, 813),
            "provenance": {
                "signature": "0" * 64, "key_id": "k1",
                "algorithm": "hmac-sha256",
                "signed_fields": ["event_id"],
            },
            "extensions": {"vendor_foo": True},
        }, "Both provenance and extensions populated."),
        ("edge-min-required-only", {
            "udm_version": SCHEMA_VERSION,
            "event_id": "e",
            "event_type": "telemetry_periodic",
            "source_id": "s",
            "source_type": "amr",
            "captured_at": "2026-06-07T19:00:00Z",
        }, "Absolute minimum: one-character required string values."),
    ]
    for name, payload, purpose in cases:
        _write_pair(folder, name, payload, {
            "purpose": purpose,
            "expected_result": "valid",
            "schema_path": EVENT_SCHEMA,
            "notes": "Edge cases must still validate against the canonical schema.",
        })
    return len(cases)


def emit_legacy() -> int:
    """Legacy fixtures are *vendored* — they are checked into the repo by
    hand at first capture and never regenerated. We still ensure the
    ``meta.yaml`` stays in sync with this script (the JSON fixture body
    is left untouched).
    """
    folder = CONFORMANCE_DIR / "legacy"
    fixture = folder / "01-phycloud-udm-event-request.json"
    src = (REPO_ROOT.parent / "PhyWare" / "PhyCloud" / "rust" / "tests"
           / "fixtures" / "udm_event_request.json")
    if not fixture.exists() and src.exists():
        folder.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(src, fixture)
    if not fixture.exists():
        return 0
    _write_yaml_only(folder, "01-phycloud-udm-event-request", {
        "purpose": (
            "Legacy fixture from PhyCloud/rust/tests/fixtures/udm_event_request.json. "
            "Captured here to track drift between historical PhyCloud test "
            "payloads and the canonical UDM schema."
        ),
        "expected_result": "drift_documented",
        "schema_path": EVENT_SCHEMA,
        "drift": [
            {"field": "udm_version", "actual": "1.0",
             "expected": "0.0.3",
             "resolution": "PhyCloud fixture to be regenerated against v0.0.3."},
            {"field": "event_type", "actual": "telemetry",
             "expected": "telemetry_periodic",
             "resolution": "Canonical event_type taxonomy uses *_periodic suffix."},
            {"field": "identity.robot_id", "actual": "amr-7",
             "expected": "identity.source_id",
             "resolution": "Rust SDK + spec normalize on source_id; "
                           "robot_id was a pre-Phase-1 alias."},
            {"field": "location.{x,y,frame}", "actual": "{x,y,frame}",
             "expected": "location.local.{x_m,y_m} + location.frame_id",
             "resolution": "Canonical schema uses explicit units (_m) and "
                           "the location.local sub-object; flat x/y/frame is "
                           "from the pre-Phase-1 location schema."},
        ],
        "notes": (
            "This fixture does NOT validate against the canonical schema "
            "and is excluded from --conformance pass criteria for the "
            "legacy/ partition. Resolution is tracked in the PhyTrace SDKs."
        ),
    })
    return 1


def _write_yaml_only(folder: Path, name: str, meta: dict[str, Any]) -> None:
    folder.mkdir(parents=True, exist_ok=True)
    with (folder / f"{name}.meta.yaml").open("w", encoding="utf-8") as fh:
        _dump_yaml(meta, fh)


def main() -> None:
    # Wipe + regenerate to keep the suite deterministic.
    # NOTE: legacy/ is excluded — legacy fixtures are vendored once and
    # never regenerated (the source PhyWare repo may not be available in
    # CI or downstream clones).
    for sub in ("valid", "invalid", "edge"):
        d = CONFORMANCE_DIR / sub
        if d.exists():
            for p in d.iterdir():
                if p.name == "README.md":
                    continue
                p.unlink()

    totals = {
        "valid_event_types": emit_valid_minimal(),
        "valid_source_types": emit_valid_source_types(),
        "valid_per_domain": emit_valid_per_domain(),
        "valid_cross_cutting": emit_valid_cross_cutting(),
        "valid_extensions": emit_valid_extensions(),
        "valid_provenance": emit_valid_provenance(),
        "invalid_envelope": emit_invalid_envelope(),
        "invalid_domain": emit_invalid_domain(),
        "invalid_provenance": emit_invalid_provenance(),
        "edge": emit_edge(),
        "legacy": emit_legacy(),
    }
    total = sum(totals.values())
    print("Generated fixtures:")
    for k, v in totals.items():
        print(f"  {k:>22}: {v}")
    print(f"  {'TOTAL':>22}: {total}")


if __name__ == "__main__":
    main()
