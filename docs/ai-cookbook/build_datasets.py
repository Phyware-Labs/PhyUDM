#!/usr/bin/env python3
"""Generate the AI cookbook datasets.

Each fixture lives at ``docs/ai-cookbook/datasets/<article>.ndjson`` and
is consumed by the cookbook's MCP / CLI walk-throughs via the
``memory`` adapter. The fixtures are deliberately small (≤ 50 events
each) so cookbook readers can ``grep`` them and reason about the
data, but rich enough that every tool call in each article returns a
meaningful result.

Re-run from the repo root::

    python3 docs/ai-cookbook/build_datasets.py
"""

from __future__ import annotations

import json
import random
from datetime import datetime, timedelta, timezone
from pathlib import Path

DATASETS_DIR = Path(__file__).resolve().parent / "datasets"
DATASETS_DIR.mkdir(parents=True, exist_ok=True)

UDM_VERSION = "0.0.3"


def event_id(n: int) -> str:
    return f"01940000-0000-7000-8000-{n:012d}"


def envelope(
    n: int,
    *,
    event_type: str,
    source_id: str,
    source_type: str,
    captured_at: datetime,
    **extras,
) -> dict:
    payload = {
        "udm_version": UDM_VERSION,
        "event_id": event_id(n),
        "event_type": event_type,
        "source_id": source_id,
        "source_type": source_type,
        "captured_at": captured_at.strftime("%Y-%m-%dT%H:%M:%SZ"),
    }
    payload.update(extras)
    return payload


def write_ndjson(name: str, events: list[dict]) -> None:
    path = DATASETS_DIR / f"{name}.ndjson"
    with path.open("w", encoding="utf-8") as fh:
        for ev in events:
            fh.write(json.dumps(ev, separators=(",", ":")) + "\n")
    print(f"  wrote {path.relative_to(DATASETS_DIR.parent.parent)} ({len(events)} events)")


# --------------------------------------------------------------------------
# Article 1 — wire-into-client. Uses the same 6-event canonical fixture as
# the eventstore conformance suite so the article's prompt actually
# matches what the user sees on disk.
# --------------------------------------------------------------------------

def build_wire_into_client() -> None:
    base = datetime(2026, 6, 7, 19, 0, 0, tzinfo=timezone.utc)
    events = [
        envelope(1, event_type="telemetry_periodic", source_id="amr-001",
                 source_type="amr", captured_at=base,
                 identity={"source_id": "amr-001", "source_type": "amr",
                           "platform": "MiR250"},
                 location={"latitude": 32.7767, "longitude": -96.7970,
                           "frame_id": "map"},
                 power={"battery": {"soc_pct": 78.4},
                        "power_consumption_w": 145.2}),
        envelope(2, event_type="telemetry_periodic", source_id="amr-001",
                 source_type="amr", captured_at=base + timedelta(seconds=2),
                 power={"battery": {"soc_pct": 78.2},
                        "power_consumption_w": 148.0}),
        envelope(3, event_type="safety_violation", source_id="amr-001",
                 source_type="amr", captured_at=base + timedelta(seconds=5),
                 safety={"safety_state": "warning", "is_safe": True,
                         "violation_count": 1}),
        envelope(4, event_type="telemetry_periodic", source_id="agv-002",
                 source_type="agv", captured_at=base + timedelta(seconds=1),
                 power={"battery": {"soc_pct": 92.1}}),
        envelope(5, event_type="emergency_stop", source_id="agv-002",
                 source_type="agv", captured_at=base + timedelta(seconds=7),
                 safety={"safety_state": "emergency_stop", "is_safe": False,
                         "e_stop": {"type": "hardware"}}),
        envelope(6, event_type="task_completed", source_id="cobot-007",
                 source_type="cobot", captured_at=base + timedelta(seconds=9),
                 manipulation={"arm_state": "home", "control_mode": "position"}),
    ]
    write_ndjson("wire-into-client", events)


# --------------------------------------------------------------------------
# Article 2 — incident reconstruction. amr-014 experiences a hardware
# e-stop at t=14:22:03 after a sequence of perception degradation +
# proximity warnings. Article walks the LLM through reconstructing it.
# --------------------------------------------------------------------------

def build_incident() -> None:
    base = datetime(2026, 3, 14, 14, 0, 0, tzinfo=timezone.utc)
    sid = "amr-014"
    events: list[dict] = []

    def at(seconds: int) -> datetime:
        return base + timedelta(seconds=seconds)

    def push(**kwargs):
        events.append(envelope(len(events) + 1, **kwargs))

    # 60 seconds of normal telemetry well before the incident.
    for i in range(0, 60, 10):
        push(event_type="telemetry_periodic", source_id=sid, source_type="amr",
             captured_at=at(i),
             motion={"speed_mps": 0.95, "motion_state": "moving"},
             power={"battery": {"soc_pct": round(75.0 - i * 0.02, 2)}},
             safety={"safety_state": "normal", "is_safe": True})

    # Sensor degradation building toward the incident at t+22:03.
    # All within the 2-minute window centred on the e-stop.
    incident_ts = base + timedelta(minutes=22, seconds=3)
    buildup_start = incident_ts - timedelta(seconds=63)

    push(event_type="error", source_id=sid, source_type="amr",
         captured_at=buildup_start,
         perception={"lidar": [{"id": "front-1", "status": "degraded"}],
                     "detection_count": 0})
    push(event_type="error", source_id=sid, source_type="amr",
         captured_at=buildup_start + timedelta(seconds=15),
         perception={"lidar": [{"id": "front-1", "status": "error"}]})

    # Proximity violation.
    push(event_type="safety_violation", source_id=sid, source_type="amr",
         captured_at=buildup_start + timedelta(seconds=40),
         safety={"safety_state": "warning",
                 "violation_count": 1,
                 "violations": [{"type": "proximity_violation",
                                 "severity": "high",
                                 "distance_m": 0.35}]})

    # Path-blocked + rerouting.
    push(event_type="path_blocked", source_id=sid, source_type="amr",
         captured_at=buildup_start + timedelta(seconds=50),
         navigation={"path": {"state": "blocked"},
                     "obstacle_count": 1,
                     "obstacles": [{"kind": "dynamic", "distance_m": 0.4}]})
    push(event_type="rerouting", source_id=sid, source_type="amr",
         captured_at=buildup_start + timedelta(seconds=55),
         navigation={"path": {"state": "planning"}})

    # The incident itself.
    push(event_type="emergency_stop", source_id=sid, source_type="amr",
         captured_at=incident_ts,
         safety={"safety_state": "emergency_stop",
                 "is_safe": False,
                 "e_stop": {"type": "hardware",
                            "triggered_at": incident_ts.isoformat().replace("+00:00", "Z")},
                 "protective_stop_active": True,
                 "protective_stop_reason": "front_lidar_failure",
                 "violation_count": 2})

    # Post-incident state
    push(event_type="state_transition", source_id=sid, source_type="amr",
         captured_at=incident_ts + timedelta(seconds=2),
         operational={"mode": "emergency", "state": "emergency_stopped",
                      "previous_state": "navigating"})
    push(event_type="system_shutdown", source_id=sid, source_type="amr",
         captured_at=incident_ts + timedelta(seconds=45),
         operational={"mode": "maintenance", "state": "shutting_down"})

    write_ndjson("incident-amr-014", events)


# --------------------------------------------------------------------------
# Article 3 — compliance audit. Q1 2026 ISO/TS 15066 (collaborative
# robots, biomechanical limits). 3 cobots × ~15 events each spanning
# collaborative-mode entry/exit, contact events, and a handful of
# violations.
# --------------------------------------------------------------------------

def build_compliance() -> None:
    base = datetime(2026, 1, 1, 8, 0, 0, tzinfo=timezone.utc)
    events: list[dict] = []
    rng = random.Random(0xC0B07)

    def push(**kwargs):
        events.append(envelope(len(events) + 1, **kwargs))

    for cobot_idx, cobot in enumerate(["cobot-101", "cobot-102", "cobot-103"]):
        platform = ["UR10e", "Franka-Panda", "Kuka-LBR-iiwa"][cobot_idx]
        for day in range(0, 90, 7):  # one cycle per week across Q1
            day_start = base + timedelta(days=day, hours=cobot_idx)

            # Collaborative mode entered.
            push(event_type="task_started", source_id=cobot, source_type="cobot",
                 captured_at=day_start,
                 identity={"source_id": cobot, "source_type": "cobot",
                           "platform": platform},
                 operational={"mode": "semi_autonomous",
                              "state": "executing_task"},
                 safety={"safety_state": "normal",
                         "collaborative_operation": {"mode": "entered",
                                                     "operator_id": f"op-{cobot_idx + 1:02d}"}})

            # Three contact events with varying force levels.
            for k in range(3):
                ts = day_start + timedelta(minutes=10 + k * 15)
                force_n = rng.uniform(60, 150)
                push(event_type="safety_violation" if force_n > 100 else "task_completed",
                     source_id=cobot, source_type="cobot",
                     captured_at=ts,
                     safety={"safety_state": "warning" if force_n > 100 else "normal",
                             "violations": ([{"type": "force_limit_exceeded",
                                              "severity": "high",
                                              "force_n": round(force_n, 1)}]
                                            if force_n > 100 else [])},
                     compliance={"functional_safety": {"performance_level": "pl_d"},
                                 "certifications": [{"name": "ISO-13849",
                                                     "status": "certified"}]})

            # Collaborative mode exited.
            push(event_type="task_completed", source_id=cobot, source_type="cobot",
                 captured_at=day_start + timedelta(hours=2),
                 safety={"safety_state": "normal"},
                 operational={"mode": "autonomous", "state": "ready"})

    write_ndjson("compliance-iso-ts-15066-q1", events)


# --------------------------------------------------------------------------
# Article 4 — fleet health Q&A. 8 robots across 3 source types, 30
# minutes of recent telemetry. Drives `query_events`, `aggregate`, and
# `timeline` tool calls.
# --------------------------------------------------------------------------

def build_fleet_health() -> None:
    base = datetime(2026, 6, 7, 12, 0, 0, tzinfo=timezone.utc)
    events: list[dict] = []
    rng = random.Random(0xF1E37)

    def push(**kwargs):
        events.append(envelope(len(events) + 1, **kwargs))

    fleet = [
        ("amr-001", "amr", 78.4),
        ("amr-002", "amr", 22.1),   # low battery
        ("amr-003", "amr", 91.5),
        ("agv-007", "agv", 64.0),
        ("agv-008", "agv", 18.5),   # low battery
        ("cobot-201", "cobot", 88.0),
        ("cobot-202", "cobot", 72.4),
        ("drone-aer-1", "drone", 56.7),
    ]

    for tick in range(0, 30 * 60, 60):  # one per minute for 30 min
        ts = base + timedelta(seconds=tick)
        for sid, stype, base_soc in fleet:
            soc = round(base_soc - tick * 0.001 + rng.uniform(-0.5, 0.5), 2)
            push(event_type="telemetry_periodic", source_id=sid, source_type=stype,
                 captured_at=ts,
                 identity={"source_id": sid, "source_type": stype,
                           "fleet_id": "warehouse-east"},
                 power={"battery": {"soc_pct": soc},
                        "power_consumption_w": round(rng.uniform(80, 200), 1)},
                 operational={"mode": "autonomous", "state": "executing_task"})

    # Inject a couple of safety events for the Q&A to find.
    push(event_type="safety_violation", source_id="agv-008", source_type="agv",
         captured_at=base + timedelta(minutes=22),
         safety={"safety_state": "warning",
                 "violation_count": 1,
                 "violations": [{"type": "speed_exceeded", "severity": "medium"}]})
    push(event_type="mode_change", source_id="amr-002", source_type="amr",
         captured_at=base + timedelta(minutes=27),
         operational={"mode": "maintenance", "state": "charging",
                      "previous_state": "executing_task"})

    write_ndjson("fleet-health-warehouse-east", events)


def main() -> None:
    build_wire_into_client()
    build_incident()
    build_compliance()
    build_fleet_health()
    print("done.")


if __name__ == "__main__":
    main()
