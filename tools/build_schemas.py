#!/usr/bin/env python3
"""Generate canonical UDM JSON Schema (Draft 2020-12) artifacts.

This is the first-pass schema generator for PhyUDM v0.0.3. It encodes the
known surface area of every domain from the Rust source-of-truth at
``PhyTrace/rust-sdk/src/models/domains/*.rs``.

For v0.0.3 the schemas are deliberately scoped to:

  * Strong typing for envelope fields + cross-domain enums.
  * Top-level domain field names + primitive types from each
    ``*Domain`` Rust struct.
  * Permissive (``additionalProperties: true``) sub-objects for nested
    types (e.g. ``LocationDomain.local``). Future PRs tighten these once
    ``schemars`` derive is wired into the Rust SDK.

Run from the repo root::

    python3 tools/build_schemas.py

This rewrites every ``schemas/v0.0.3/**/*.schema.json`` in place. The
generated JSON is the canonical artifact — third-party validators consume
the JSON, not this script.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

VERSION = "0.0.3"
SCHEMA_HOST = "https://schemas.phyudm.org"
BASE_ID = f"{SCHEMA_HOST}/v{VERSION}"
DRAFT = "https://json-schema.org/draft/2020-12/schema"

REPO_ROOT = Path(__file__).resolve().parent.parent
OUT_DIR = REPO_ROOT / "schemas" / f"v{VERSION}"

# ---------------------------------------------------------------------------
# Enums — sourced from PhyTrace/rust-sdk/src/models/enums.rs.
# Variant names are emitted in snake_case (Rust ``#[serde(rename_all =
# "snake_case")]``); explicit overrides via ``#[serde(rename)]`` are
# captured below.
# ---------------------------------------------------------------------------

ENUMS: dict[str, list[str]] = {
    "EventType": [
        "telemetry_periodic", "telemetry_on_change", "telemetry_snapshot",
        "state_transition", "mode_change",
        "task_started", "task_completed", "task_failed", "task_cancelled",
        "goal_reached", "path_blocked", "rerouting",
        "safety_violation", "emergency_stop",
        "system_startup", "system_shutdown",
        "error", "custom",
    ],
    "SourceType": [
        "amr", "agv", "autonomous_forklift", "delivery_robot",
        "cleaning_robot", "inspection_robot", "security_robot",
        "industrial_arm", "cobot", "mobile_manipulator",
        "autonomous_vehicle", "electric_vehicle", "drone",
        "humanoid", "quadruped", "human", "custom", "simulation",
    ],
    "OperationalMode": [
        "autonomous", "manual", "semi_autonomous", "teleoperated",
        "learning", "maintenance", "emergency", "idle",
    ],
    "OperationalState": [
        "off", "booting", "ready", "executing_task", "navigating",
        "manipulating", "charging", "paused", "error",
        "emergency_stopped", "shutting_down", "recovering",
    ],
    "ErrorSeverity": ["info", "warning", "error", "critical", "fatal"],
    "MotionState": [
        "stationary", "moving", "accelerating", "decelerating",
        "rotating", "reversing",
    ],
    "BatteryChemistry": [
        "li_fe_po4", "nmc", "li_co_o2", "lmo", "lead_acid",
        "ni_mh", "solid_state", "other",
    ],
    "ChargingState": [
        "not_charging", "preparing", "charging", "complete", "error",
    ],
    "PowerSource": ["battery", "external", "solar", "fuel_cell", "hybrid"],
    "SafetyState": [
        "normal", "warning", "protective_stop", "emergency_stop",
        "safety_interlock", "reduced_speed",
    ],
    "EStopType": ["software", "hardware", "remote", "automatic"],
    "SafetyZoneType": [
        "warning", "slowdown", "stop", "restricted", "collaborative",
    ],
    "ViolationSeverity": ["low", "medium", "high", "critical"],
    "ViolationType": [
        "speed_exceeded", "zone_violation", "proximity_violation",
        "collision_detected", "stability_violation",
        "force_limit_exceeded", "communication_loss", "sensor_failure",
    ],
    "LocalizationQuality": ["excellent", "good", "fair", "poor", "lost"],
    "PathState": [
        "none", "planning", "valid", "executing", "blocked",
        "completed", "failed",
    ],
    "ObstacleType": [
        "unknown", "static", "dynamic", "human", "robot",
        "vehicle", "temporary",
    ],
    "SensorStatus": ["ok", "degraded", "error", "offline", "calibrating"],
    "GpsFixType": [
        "no_fix", "fix2d", "fix3d", "dgps", "rtk_float", "rtk_fixed",
    ],
    "DetectionConfidence": ["low", "medium", "high", "very_high"],
    "MotorStatus": [
        "ok", "overheating", "overcurrent", "error", "disabled",
    ],
    "GripperState": ["open", "closed", "holding", "moving", "error"],
    "LiftState": ["lowered", "raised", "moving", "intermediate", "error"],
    "NetworkType": [
        "wifi", "ethernet", "cellular", "lora", "bluetooth", "mesh",
    ],
    "ConnectionStatus": ["connected", "disconnected", "connecting", "error"],
    "CellularGeneration": ["3g", "4g", "5g"],
    "LoadStatus": ["empty", "partial_load", "full_load", "overloaded"],
    "ArmState": [
        "home", "moving", "at_target", "grasping", "placing",
        "avoiding", "error",
    ],
    "GraspPhase": [
        "idle", "approaching", "pre_grasp", "closing", "secured",
        "lifting", "failed",
    ],
    "ManipulationControlMode": [
        "position", "velocity", "force", "impedance", "hybrid",
    ],
    "InteractionState": [
        "none", "human_detected", "awaiting_input", "interacting",
        "handover", "following",
    ],
    "HandoverState": [
        "none", "offering", "receiving", "complete", "aborted",
    ],
    "HumanActivity": [
        "unknown", "standing", "walking", "running", "sitting",
        "working", "gesturing",
    ],
    "FleetRole": [
        "independent", "leader", "follower", "swarm_member", "coordinator",
    ],
    "FormationType": [
        "none", "line", "column", "vee", "diamond", "circle", "custom",
    ],
    "TrafficPriority": ["low", "normal", "high", "emergency"],
    "SimulatorType": [
        "gazebo", "isaac_sim", "unity", "unreal", "webots",
        "custom_python", "other",
    ],
    "SimulationFidelity": [
        "low", "medium", "high", "physics_accurate",
    ],
    "ThermalState": ["normal", "elevated", "high", "critical", "cold"],
    "CoolingMode": ["passive", "active", "liquid", "maximum"],
    "SoundType": [
        "unknown", "speech", "alarm", "impact", "machine", "emergency",
    ],
    "DoorState": [
        "open", "closed", "opening", "closing", "locked", "error",
    ],
    "ElevatorState": [
        "idle", "called", "arriving", "doors_open", "boarding",
        "in_transit", "arrived",
    ],
    "ChargingStationState": [
        "available", "occupied", "out_of_service", "reserved",
    ],
    "CertificationStatus": [
        "not_certified", "pending", "certified", "expired",
    ],
    "SafetyIntegrityLevel": [
        "none", "pl_a", "pl_b", "pl_c", "pl_d", "pl_e",
    ],
    "ModelType": [
        "object_detection", "semantic_segmentation",
        "instance_segmentation", "pose_estimation", "depth_estimation",
        "navigation", "anomaly_detection", "speech_recognition",
        "nlu", "custom",
    ],
    "InferenceDevice": ["cpu", "gpu", "npu", "edge_tpu", "fpga"],
    "AnomalySeverity": ["low", "medium", "high", "critical"],
    "ComponentHealth": [
        "healthy", "degraded", "needs_attention", "failed", "unknown",
    ],
    "MaintenanceUrgency": [
        "none", "routine", "soon", "urgent", "critical",
    ],
}


def enum_schema(name: str) -> dict[str, Any]:
    """Reference an enum by name as an inline schema fragment."""
    return {"type": "string", "enum": ENUMS[name]}


# ---------------------------------------------------------------------------
# Primitive helpers.
# ---------------------------------------------------------------------------

OBJECT_REF_REF = "object_ref.schema.json"


def f(desc: str | None = None, **extra: Any) -> dict[str, Any]:
    out: dict[str, Any] = {"type": "number"}
    if desc:
        out["description"] = desc
    out.update(extra)
    return out


def i(desc: str | None = None, **extra: Any) -> dict[str, Any]:
    out: dict[str, Any] = {"type": "integer"}
    if desc:
        out["description"] = desc
    out.update(extra)
    return out


def s(desc: str | None = None, **extra: Any) -> dict[str, Any]:
    out: dict[str, Any] = {"type": "string"}
    if desc:
        out["description"] = desc
    out.update(extra)
    return out


def b(desc: str | None = None) -> dict[str, Any]:
    out: dict[str, Any] = {"type": "boolean"}
    if desc:
        out["description"] = desc
    return out


def dt(desc: str | None = None) -> dict[str, Any]:
    out: dict[str, Any] = {"type": "string", "format": "date-time"}
    if desc:
        out["description"] = desc
    return out


def arr(items: dict[str, Any], desc: str | None = None) -> dict[str, Any]:
    out: dict[str, Any] = {"type": "array", "items": items}
    if desc:
        out["description"] = desc
    return out


def obj_open(desc: str | None = None) -> dict[str, Any]:
    """Open-ended nested object (forward-compat for un-schematized sub-types)."""
    out: dict[str, Any] = {"type": "object", "additionalProperties": True}
    if desc:
        out["description"] = desc
    return out


# ---------------------------------------------------------------------------
# Domain definitions — mirror PhyTrace/rust-sdk/src/models/domains/*.rs.
# Property keys match Rust ``#[serde(rename_all = "snake_case")]`` output.
# ---------------------------------------------------------------------------

DOMAINS: list[tuple[int, str, str, dict[str, Any]]] = [
    (1, "identity", "Identity", {
        "title": "Identity Domain",
        "description": (
            "Who/what is reporting: stable identifiers, hardware/firmware "
            "metadata, organizational placement."
        ),
        "properties": {
            "source_id": s("Stable source identifier."),
            "source_type": enum_schema("SourceType"),
            "platform": s("Vendor platform/product name."),
            "model": s("Model designation."),
            "serial_number": s(),
            "firmware_version": s(),
            "hardware_version": s(),
            "fleet_id": s(),
            "site_id": s(),
            "organization_id": s(),
            "display_name": s(),
            "tags": arr(s()),
            "metadata": {
                "type": "object",
                "description": "Free-form string→string metadata.",
                "additionalProperties": {"type": "string"},
            },
            "mac_address": s(pattern=r"^([0-9A-Fa-f]{2}[:-]){5}[0-9A-Fa-f]{2}$"),
            "ip_address": s(),
        },
    }),
    (2, "location", "Location", {
        "title": "Location Domain",
        "description": "Where the source is — geo, local, semantic, accuracy.",
        "properties": {
            "latitude": f(minimum=-90, maximum=90),
            "longitude": f(minimum=-180, maximum=180),
            "altitude_m": f(),
            "heading_deg": f(minimum=0, exclusiveMaximum=360),
            "local": obj_open("Local-frame coordinates (x_m, y_m, z_m, ...)."),
            "grid": obj_open("Grid-cell coordinates."),
            "semantic": obj_open("Semantic location (room/zone/poi)."),
            "horizontal_accuracy_m": f(minimum=0),
            "vertical_accuracy_m": f(minimum=0),
            "covariance": obj_open("Covariance matrix entries."),
            "frame_id": s("Coordinate frame identifier (e.g. 'map', 'odom')."),
            "map_id": s(),
            "floor": i(),
        },
    }),
    (3, "motion", "Motion", {
        "title": "Motion Domain",
        "description": "Velocities, accelerations, odometry, motion state.",
        "properties": {
            "speed_mps": f(minimum=0),
            "linear_velocity": obj_open(),
            "angular_velocity": obj_open(),
            "linear_acceleration": obj_open(),
            "angular_acceleration": obj_open(),
            "odometry": obj_open(),
            "motion_state": enum_schema("MotionState"),
            "frame_id": s(),
            "commanded_linear_mps": f(),
            "commanded_angular_dps": f(),
        },
    }),
    (4, "power", "Power", {
        "title": "Power Domain",
        "description": "Battery, charging, power consumption, runtime estimate.",
        "properties": {
            "battery": obj_open("Battery state-of-charge/health/voltage/etc."),
            "charging": obj_open("Charging session details."),
            "power_consumption_w": f(minimum=0),
            "average_power_w": f(minimum=0),
            "power_source": enum_schema("PowerSource"),
            "voltage_v": f(),
            "current_a": f(),
            "estimated_runtime_min": f(minimum=0),
            "estimated_range_m": f(minimum=0),
        },
    }),
    (5, "operational", "Operational", {
        "title": "Operational State Domain",
        "description": "Mode, state, current task, errors, uptime.",
        "properties": {
            "mode": enum_schema("OperationalMode"),
            "state": enum_schema("OperationalState"),
            "previous_state": enum_schema("OperationalState"),
            "time_in_state_sec": f(minimum=0),
            "task": obj_open("Current task description."),
            "queue": obj_open("Pending task queue."),
            "errors": arr(obj_open("Operational error entry.")),
            "error_count": i(minimum=0),
            "is_available": b(),
            "unavailable_reason": s(),
            "uptime_sec": f(minimum=0),
            "last_boot": dt(),
            "mission_id": s(),
            "session_id": s(),
        },
    }),
    (6, "navigation", "Navigation", {
        "title": "Navigation Domain",
        "description": "Localization, SLAM, path, goal, obstacle awareness.",
        "properties": {
            "localization": obj_open(),
            "slam": obj_open(),
            "path": obj_open(),
            "goal": obj_open(),
            "obstacles": arr(obj_open()),
            "obstacle_count": i(minimum=0),
            "semantic_map": obj_open(),
            "costmap": obj_open(),
        },
    }),
    (7, "perception", "Perception", {
        "title": "Perception Domain",
        "description": "Sensor stack, fused detections, environment perception.",
        "properties": {
            "lidar": arr(obj_open()),
            "cameras": arr(obj_open()),
            "imu": obj_open(),
            "gps": obj_open(),
            "uwb": obj_open(),
            "detections": arr(obj_open()),
            "detection_count": i(minimum=0),
            "environment": obj_open(),
            "ultrasonics": arr(obj_open()),
            "bumpers": arr(obj_open()),
            "cliff_sensors": arr(obj_open()),
        },
    }),
    (8, "safety", "Safety", {
        "title": "Safety Domain",
        "description": "Safety state, e-stop, zones, violations, collaborative ops.",
        "properties": {
            "safety_state": enum_schema("SafetyState"),
            "is_safe": b(),
            "e_stop": obj_open(),
            "protective_stop_active": b(),
            "protective_stop_reason": s(),
            "zones": arr(obj_open()),
            "current_zone_ids": arr(s()),
            "proximity": obj_open(),
            "violations": arr(obj_open()),
            "violation_count": i(minimum=0),
            "collaborative_operation": obj_open(),
            "speed_limit_mps": f(minimum=0),
            "speed_limit_reason": s(),
            "safety_system_ok": b(),
            "safety_plc_status": s(),
            "last_safety_check": dt(),
        },
    }),
    (9, "actuators", "Actuators", {
        "title": "Actuators Domain",
        "description": "Drive motors, joints, grippers, lifts, steering.",
        "properties": {
            "drive_motors": arr(obj_open()),
            "joints": arr(obj_open()),
            "grippers": arr(obj_open()),
            "lifts": arr(obj_open()),
            "steering": arr(obj_open()),
            "hydraulics": obj_open(),
            "pneumatics": obj_open(),
        },
    }),
    (10, "communication", "Communication", {
        "title": "Communication Domain",
        "description": "Network, cellular, fleet comms, integrations, PhyTrace conn.",
        "properties": {
            "network": obj_open(),
            "cellular": obj_open(),
            "fleet": obj_open(),
            "integrations": arr(obj_open()),
            "phytrace": obj_open(),
        },
    }),
    (11, "compute", "Compute", {
        "title": "Compute Domain",
        "description": "CPU, memory, GPU, storage, ROS status, host metadata.",
        "properties": {
            "cpu": obj_open(),
            "memory": obj_open(),
            "gpu": arr(obj_open()),
            "storage": arr(obj_open()),
            "processes": obj_open(),
            "ros": obj_open(),
            "uptime_sec": f(minimum=0),
            "load_average": {
                "type": "array",
                "items": {"type": "number", "minimum": 0},
                "minItems": 3,
                "maxItems": 3,
                "description": "1/5/15-minute load average triple.",
            },
            "kernel_version": s(),
            "os_version": s(),
        },
    }),
    (12, "ai", "Ai", {
        "title": "AI / Reasoning Domain",
        "description": "Deployed models, decisions, anomalies, inference stats.",
        "properties": {
            "models": arr(obj_open()),
            "decisions": arr(obj_open()),
            "anomalies": arr(obj_open()),
            "total_inferences": i(minimum=0),
            "avg_inference_ms": f(minimum=0),
        },
    }),
    (13, "maintenance", "Maintenance", {
        "title": "Maintenance Domain",
        "description": "Health score, component status, diagnostics, schedule.",
        "properties": {
            "health_score": f(minimum=0, maximum=1),
            "components": arr(obj_open()),
            "diagnostics": arr(obj_open()),
            "maintenance_due": arr(obj_open()),
            "last_maintenance": dt(),
            "next_maintenance": dt(),
            "operating_hours": f(minimum=0),
            "urgency": enum_schema("MaintenanceUrgency"),
        },
    }),
    (14, "context", "Context", {
        "title": "Context Domain",
        "description": "Time-of-day, facility, weather, traffic context.",
        "properties": {
            "time": obj_open(),
            "facility": obj_open(),
            "weather": obj_open(),
            "traffic": obj_open(),
        },
    }),
    (15, "payload", "Payload", {
        "title": "Payload / Cargo Domain",
        "description": "Load status, weight, items, compartments, securement.",
        "properties": {
            "load_status": enum_schema("LoadStatus"),
            "total_weight_kg": f(minimum=0),
            "max_weight_kg": f(minimum=0),
            "weight_utilization_pct": f(minimum=0, maximum=100),
            "items": arr(obj_open()),
            "item_count": i(minimum=0),
            "compartments": arr(obj_open()),
            "is_secured": b(),
            "center_of_mass_offset_m": {
                "type": "array",
                "items": {"type": "number"},
                "minItems": 3,
                "maxItems": 3,
                "description": "Center-of-mass offset (x, y, z) in meters.",
            },
        },
    }),
    (16, "manipulation", "Manipulation", {
        "title": "Manipulation Domain",
        "description": "Arm/end-effector state, workspace, grasp, hand-guiding.",
        "properties": {
            "arm_state": enum_schema("ArmState"),
            "end_effector": obj_open(),
            "workspace": obj_open(),
            "grasp": obj_open(),
            "hand_guiding": obj_open(),
            "control_mode": enum_schema("ManipulationControlMode"),
            "collision_avoidance_active": b(),
        },
    }),
    (17, "hri", "Hri", {
        "title": "Human-Robot Interaction Domain",
        "description": "Interaction state, tracked humans, voice, handover, social.",
        "properties": {
            "interaction_state": enum_schema("InteractionState"),
            "tracked_humans": arr(obj_open()),
            "human_count": i(minimum=0),
            "voice": obj_open(),
            "handover": obj_open(),
            "social": obj_open(),
            "gestures": arr(obj_open()),
        },
    }),
    (18, "coordination", "Coordination", {
        "title": "Multi-Agent Coordination Domain",
        "description": "Fleet role, formations, neighbors, swarm/traffic.",
        "properties": {
            "fleet_role": enum_schema("FleetRole"),
            "formation": obj_open(),
            "neighbors": arr(obj_open()),
            "swarm": obj_open(),
            "traffic": obj_open(),
        },
    }),
    (19, "simulation", "Simulation", {
        "title": "Simulation / Digital Twin Domain",
        "description": "Sim flag, simulator info, scenario, digital twin link.",
        "properties": {
            "is_simulated": b(),
            "simulator": obj_open(),
            "scenario": obj_open(),
            "digital_twin": obj_open(),
            "sim_time": dt(),
            "time_scale": f(minimum=0),
            "step_count": i(minimum=0),
        },
    }),
    (20, "thermal", "Thermal", {
        "title": "Thermal Management Domain",
        "description": "Thermal state, per-component temps, cooling/heating.",
        "properties": {
            "thermal_state": enum_schema("ThermalState"),
            "components": arr(obj_open()),
            "ambient_temp_c": f(),
            "cooling_system": obj_open(),
            "heating_system": obj_open(),
            "is_throttling": b(),
        },
    }),
    (21, "audio", "Audio", {
        "title": "Audio Domain",
        "description": "Microphones, speakers, detected sounds, voice output.",
        "properties": {
            "microphones": arr(obj_open()),
            "speakers": arr(obj_open()),
            "sound_detection": arr(obj_open()),
            "voice_output": obj_open(),
            "ambient_noise_db": f(),
        },
    }),
    (22, "environment_interaction", "EnvironmentInteraction", {
        "title": "Environment Interaction Domain",
        "description": "Doors, elevators, charging stations, surface contact.",
        "properties": {
            "doors": arr(obj_open()),
            "elevators": arr(obj_open()),
            "charging_stations": arr(obj_open()),
            "surface": obj_open(),
        },
    }),
    (23, "compliance", "Compliance", {
        "title": "Compliance / Certification Domain",
        "description": "Certifications, functional safety, cybersecurity, regulatory.",
        "properties": {
            "certifications": arr(obj_open()),
            "functional_safety": obj_open(),
            "cybersecurity": obj_open(),
            "regulatory": obj_open(),
        },
    }),
]


# ---------------------------------------------------------------------------
# Envelope, event, object_ref.
# ---------------------------------------------------------------------------

def envelope_schema() -> dict[str, Any]:
    return {
        "$schema": DRAFT,
        "$id": f"{BASE_ID}/envelope.schema.json",
        "title": "UDM Envelope",
        "description": (
            "Required + optional envelope fields shared by every UDM event. "
            "Domain payloads are layered on top of the envelope in "
            "``event.schema.json``."
        ),
        "type": "object",
        "required": [
            "udm_version", "event_id", "event_type",
            "source_id", "source_type", "captured_at",
        ],
        "properties": {
            "udm_version": s(
                "Schema version this event conforms to (SemVer).",
                pattern=r"^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$",
            ),
            "event_id": s(
                "Globally unique event identifier; UUIDv7 recommended.",
                minLength=1,
            ),
            "event_type": enum_schema("EventType"),
            "source_id": s("Unique identifier of the emitting source.", minLength=1),
            "source_type": enum_schema("SourceType"),
            "captured_at": dt("ISO-8601 timestamp when data was captured at source."),
            "received_at": dt("ISO-8601 timestamp when ingest layer received the event."),
            "sent_at": dt("ISO-8601 timestamp when source sent the event."),
            "sequence_num": i(
                "Monotonic sequence number from source.", minimum=0,
            ),
            "session_id": s("Session/run identifier for grouping events."),
            "sdk_version": s(
                "Optional emitting SDK version. Rust SDK populates this; "
                "third-party emitters MAY omit it."
            ),
        },
        "additionalProperties": True,
    }


def object_ref_schema() -> dict[str, Any]:
    return {
        "$schema": DRAFT,
        "$id": f"{BASE_ID}/{OBJECT_REF_REF}",
        "title": "Object Reference",
        "description": (
            "Cross-domain reference to a first-class entity. ``kind`` is the "
            "referenced object class (e.g. ``task``, ``zone``, ``map``, "
            "``component``); ``id`` is the stable string identifier."
        ),
        "type": "object",
        "required": ["kind", "id"],
        "properties": {
            "kind": s("Referenced object class.", minLength=1),
            "id": s("Referenced object identifier.", minLength=1),
            "source_id": s("Optional originating source identifier."),
        },
        "additionalProperties": False,
    }


def provenance_schema_fragment() -> dict[str, Any]:
    return {
        "type": "object",
        "description": "Event-level integrity / provenance metadata.",
        "properties": {
            "signature": s("HMAC-SHA256 or similar signature value."),
            "key_id": s("Identifier of the signing key."),
            "algorithm": s("Signature algorithm (e.g. 'hmac-sha256')."),
            "signed_fields": arr(s(), "Fields included in the signature."),
            "signed_at": dt(),
        },
        "additionalProperties": True,
    }


def event_schema() -> dict[str, Any]:
    domain_refs: dict[str, Any] = {}
    for _, key, _, _ in DOMAINS:
        domain_refs[key] = {"$ref": f"domains/{_domain_filename(key)}"}
    return {
        "$schema": DRAFT,
        "$id": f"{BASE_ID}/event.schema.json",
        "title": "UDM Event",
        "description": (
            "A complete UDM event: envelope fields + optional payloads for "
            "each of the 23 UDM domains + provenance + free-form extensions."
        ),
        "allOf": [{"$ref": "envelope.schema.json"}],
        "type": "object",
        "properties": {
            **domain_refs,
            "provenance": provenance_schema_fragment(),
            "extensions": {
                "type": "object",
                "description": (
                    "Free-form extension payload. Vendor-specific keys SHOULD "
                    "be prefixed with ``custom_`` or the vendor namespace; see "
                    "spec/appendix-b.md."
                ),
                "additionalProperties": True,
            },
        },
        "additionalProperties": True,
    }


def _domain_filename(key: str) -> str:
    # 01-identity.schema.json etc.
    for n, k, _, _ in DOMAINS:
        if k == key:
            slug = k.replace("_", "-")
            return f"{n:02d}-{slug}.schema.json"
    raise KeyError(key)


def _domain_filename_by_index(idx: int) -> str:
    for n, k, _, _ in DOMAINS:
        if n == idx:
            slug = k.replace("_", "-")
            return f"{n:02d}-{slug}.schema.json"
    raise KeyError(idx)


def domain_schema(num: int, key: str, _camel: str, body: dict[str, Any]) -> dict[str, Any]:
    fname = _domain_filename_by_index(num)
    return {
        "$schema": DRAFT,
        "$id": f"{BASE_ID}/domains/{fname}",
        "title": body["title"],
        "description": body["description"],
        "type": "object",
        "properties": body["properties"],
        # First-pass forward compatibility: tolerate unknown fields. Future
        # PRs tighten with `additionalProperties: false` once schemars-derived
        # schemas land via the `udm-rust-sdk-consume` task.
        "additionalProperties": True,
    }


# ---------------------------------------------------------------------------
# Writer.
# ---------------------------------------------------------------------------

def _write(path: Path, schema: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as fh:
        json.dump(schema, fh, indent=2, sort_keys=False)
        fh.write("\n")


def main() -> None:
    _write(OUT_DIR / "envelope.schema.json", envelope_schema())
    _write(OUT_DIR / "object_ref.schema.json", object_ref_schema())
    _write(OUT_DIR / "event.schema.json", event_schema())
    for num, key, camel, body in DOMAINS:
        fname = _domain_filename_by_index(num)
        _write(OUT_DIR / "domains" / fname, domain_schema(num, key, camel, body))
    print(f"Wrote {2 + 1 + len(DOMAINS)} schemas to {OUT_DIR}/")


if __name__ == "__main__":
    main()
