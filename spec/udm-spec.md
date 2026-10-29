# Unified Data Model (UDM) Specification

**Version:** 0.0.3  
**Status:** Draft  
**Date:** January 12, 2026

---

## Changelog

### Version 0.0.3 (January 12, 2026)

**Collaborative Robot Safety:**
- **Safety Domain (8):** Added `collaborative_operation` section covering:
  - `operation_mode`: Collaborative operation modes per ANSI/RIA R15.06 (safety_rated_monitored_stop, speed_separation_monitoring, power_force_limiting, hand_guiding)
  - `safety_config_checksum`: Unique identifier for safety parameter verification per RIA TR R15.606
  - `speed_separation`: Dynamic protective separation distance monitoring with speed limits
  - `power_force_limits`: Maximum force/power values per ISO/TS 15066
  - `contact_event`: Quasi-static vs transient contact classification with biomechanical limits
  - `body_region_limits`: Per-body-region force/pressure thresholds per ISO/TS 15066 Annex A
- **Operational Domain (5):** Added `collaborative` to operational mode enum
- **Manipulation Domain (16):** Added `hand_guiding` control mode and hand guiding state tracking
- **Event Types:** Added collaborative-specific events:
  - `safety.collaborative_mode_entered`, `safety.collaborative_mode_exited`
  - `safety.contact_detected`, `safety.force_limit_exceeded`
  - `safety.separation_violated`, `safety.config_checksum_mismatch`
  - `manipulation.hand_guiding_started`, `manipulation.hand_guiding_completed`

**Standards Compliance:**
- Enhanced support for ANSI/RIA R15.06 and RIA TR R15.606 collaborative robot requirements
- ISO/TS 15066 biomechanical limits and contact event classification
- Safety-rated monitored stop, speed and separation monitoring, power and force limiting

### Version 0.0.2 (January 2, 2026)

**New Sections:**
- **Object References:** Added canonical `object_ref` schema for consistent object identity across domains

**ID Semantics:**
- `detection_id`: Ephemeral, per-frame perception output
- `tracking_id`: Short-lived perception track (may change after re-ID)
- `object_id`: Session-stable ID linking perception → manipulation → payload
- `item_id`: External business identifier (WMS, ERP, asset management)

**Updated Domains:**
- **Perception (7):** `detections[]` now supports `object_id`, `tracking_id`, `object_type`, `dimensions_m`, `pose`, `frame_id`
- **Payload/Cargo (15):** `items[]` now includes `object_id` for cross-domain linking; `item_id` clarified as external business identifier
- **Manipulation (16):** `grasp` now includes `object_id`, `item_id`, `object_type`, `object_class`, `tracking_id`
- **HRI (17):** `handover` now includes `object_id`, `item_id`, `object_type`, `object_class`

### Version 0.0.1 (January 2, 2026)

**New Domains:**
- **Payload/Cargo Domain (15):** Cargo management, cold chain, item tracking
- **Manipulation Domain (16):** Robot arm state, end-effectors, motion planning, grasping
- **Human-Robot Interaction Domain (17):** HRI events, gesture recognition, voice commands, handover
- **Multi-Agent Coordination Domain (18):** Fleet roles, formations, swarm behavior, traffic management
- **Simulation/Digital Twin Domain (19):** Simulation parameters, scenario tracking, digital twin sync
- **Thermal Management Domain (20):** Component temperatures, cooling/heating systems
- **Audio Domain (21):** Sound detection, voice I/O, acoustic monitoring
- **Environment Interaction Domain (22):** Doors, elevators, charging stations, surfaces
- **Compliance/Certification Domain (23):** Regulatory certifications, functional safety, cybersecurity

**New Source Types:**
- `swarm_robot`, `exoskeleton`, `telepresence_robot`, `cleaning_robot`, `hospitality_robot`
- `retail_robot`, `mining_robot`, `forestry_robot`, `space_robot`, `underwater_robot`

**New Event Types:**
- `interaction.*` events for HRI
- `payload.*` events for cargo
- `coordination.*` events for multi-agent
- `manipulation.*` events for arm/gripper
- `environment.*` events for doors/elevators

**Enhanced Domains:**
- **Navigation:** Added SLAM status, semantic mapping, multi-floor navigation, planner configuration
- **Perception:** Added thermal cameras, event cameras, tactile sensors, UWB, magnetometer, barometer, wheel encoders, semantic segmentation
- **Actuators:** Added suction, steering, hydraulics, pneumatics, propellers, tracks, legs, wheels

**New Sections:**
- Data Quality & Validation guidelines
- Streaming & Batching specifications
- Vendor Extension Registry (Appendix B)
- Units Reference (Appendix C)

### Version 0.0.1 (January 2, 2026)
- Initial specification with 15 core domains
- Core envelope and event types
- Basic source types for common robot categories

---

## Overview

The Unified Data Model (UDM) provides a comprehensive, platform-agnostic schema for representing telemetry and events from any autonomous system. The UDM is designed to:

1. **Normalize** diverse robot telemetry into a consistent, queryable format
2. **Preserve** original platform-specific data for debugging and audit
3. **Enable** safety rules, compliance checking, and analytics across all robot types
4. **Support** vendor extensions without schema conflicts
5. **Scale** from simple AMRs to complex multi-modal autonomous vehicles

---

## Design Principles

| Principle | Description |
|-----------|-------------|
| **Canonical Fields First** | Common robotics concepts (position, velocity, battery) have standard field names |
| **Hierarchical Domains** | Related fields are grouped into logical domains (identity, motion, power, etc.) |
| **Optional Everything** | No field is required except `event_id`, `source_id`, and `captured_at` |
| **Extensible by Design** | `extensions` namespace allows vendor/platform-specific data |
| **Immutable Events** | Each UDM record is immutable once created; updates create new events |
| **SI Units** | All physical quantities use SI units with explicit unit fields where ambiguous |
| **Temporal Precision** | Timestamps use ISO 8601 with microsecond precision and timezone |

---

## Object References

Physical objects (packages, tools, pallets, parts, etc.) that robots perceive, manipulate, or transport are referenced consistently across domains using the `object_ref` pattern.

### ID Semantics

| ID Type | Scope | Description |
|---------|-------|-------------|
| `detection_id` | Single frame | Unique ID for one perception output (ephemeral, per-frame) |
| `tracking_id` | Perception track | Short-lived ID for a tracked object; may change after occlusion/re-identification |
| `object_id` | Session | Stable ID for a physical object within a `session_id`; links perception, manipulation, and payload |
| `item_id` | External system | Business identifier from WMS, ERP, or asset management (e.g., order number, SKU, asset tag) |

### object_ref Schema

When referencing a physical object, use the following structure:

```json
{
  "object_id": "obj-123",
  "item_id": "ORD-2026-00456",
  "object_type": "package",
  "object_class": "cardboard_box",
  "dimensions_m": { "length": 0.4, "width": 0.3, "height": 0.2 },
  "mass_kg": 2.5,
  "tracking_id": "track-001",
  "detection_confidence": 0.95,
  "barcode": "1234567890123",
  "rfid_tag": "RFID-ABC-123",
  "asset_id": "ASSET-00789",
  "hazards": ["fragile"],
  "pose": {
    "x_m": 1.2,
    "y_m": 0.5,
    "z_m": 0.8,
    "roll_deg": 0.0,
    "pitch_deg": 0.0,
    "yaw_deg": 45.0
  },
  "frame_id": "world"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `object_id` | string | Session-stable ID linking perception → manipulation → payload |
| `item_id` | string | External business identifier (WMS order, SKU, asset tag) |
| `object_type` | string | High-level type: `package`, `pallet`, `tote`, `bin`, `tool`, `part`, `container`, `unknown` |
| `object_class` | string | Specific classification from perception (e.g., `cardboard_box`, `plastic_tote`) |
| `dimensions_m` | object | Physical dimensions: `length`, `width`, `height` |
| `mass_kg` | float | Known or estimated mass |
| `tracking_id` | string | Current perception tracking ID (may change) |
| `detection_confidence` | float | Confidence score from perception (0.0–1.0) |
| `barcode` | string | Barcode value if scanned |
| `rfid_tag` | string | RFID tag ID if detected |
| `asset_id` | string | Asset management system ID |
| `hazards` | array[string] | Hazard labels: `fragile`, `flammable`, `corrosive`, `heavy`, `temperature_sensitive` |
| `pose` | object | Object pose if known (position + orientation) |
| `frame_id` | string | Reference frame for pose |

**Usage:** Not all fields are required. Include `object_id` when linking across domains. Include `item_id` when correlating with business systems.

---

## Schema Version & Compatibility

```json
{
  "udm_version": "0.0.3",
  "udm_schema": "https://schemas.phyudm.org/v0.0.3"
}
```

**Versioning Policy:**
- **Patch (1.0.x):** Backward-compatible additions (new optional fields)
- **Minor (1.x.0):** Backward-compatible domain additions
- **Major (x.0.0):** Breaking changes (field renames, type changes, removals)

---

## Core Envelope

Every UDM event is wrapped in a standard envelope:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `udm_version` | string | Yes | Schema version (e.g., "0.0.1") |
| `event_id` | string | Yes | Globally unique event identifier (UUID v7 recommended) |
| `event_type` | string | Yes | Event classification (see Event Types) |
| `source_id` | string | Yes | Unique identifier of the data source (robot, sensor, system) |
| `source_type` | string | Yes | Source classification (see Source Types) |
| `captured_at` | string | Yes | ISO 8601 timestamp when data was captured at source |
| `received_at` | string | No | ISO 8601 timestamp when data was received by the ingest layer |
| `sent_at` | string | No | ISO 8601 timestamp when data was sent from source |
| `sequence_num` | integer | No | Monotonic sequence number from source |
| `session_id` | string | No | Session/run identifier for grouping events |

---

## Event Types

| Event Type | Description |
|------------|-------------|
| `telemetry.periodic` | Regular interval telemetry snapshot |
| `telemetry.on_change` | Telemetry emitted on significant change |
| `state.transition` | Operational state change |
| `safety.violation` | Safety rule violation detected |
| `safety.warning` | Safety threshold approaching |
| `safety.e_stop` | Emergency stop triggered |
| `safety.collaborative_mode_entered` | Entered collaborative operation mode |
| `safety.collaborative_mode_exited` | Exited collaborative operation mode |
| `safety.contact_detected` | Physical contact with human detected |
| `safety.force_limit_exceeded` | Force/power limit exceeded during contact |
| `safety.separation_violated` | Minimum protective separation distance violated |
| `safety.config_checksum_mismatch` | Safety configuration checksum verification failed |
| `task.started` | Task execution began |
| `task.completed` | Task execution completed |
| `task.failed` | Task execution failed |
| `task.cancelled` | Task execution cancelled |
| `navigation.goal_reached` | Navigation goal achieved |
| `navigation.path_blocked` | Path obstruction detected |
| `navigation.rerouting` | Path replanning initiated |
| `sensor.degraded` | Sensor performance degraded |
| `sensor.failed` | Sensor failure detected |
| `sensor.recovered` | Sensor recovered from failure |
| `power.low_battery` | Battery below threshold |
| `power.charging_started` | Charging session began |
| `power.charging_completed` | Charging session completed |
| `maintenance.required` | Maintenance action needed |
| `maintenance.performed` | Maintenance action completed |
| `communication.connected` | Network/fleet connection established |
| `communication.disconnected` | Network/fleet connection lost |
| `ai.decision` | AI/ML model decision trace |
| `ai.intervention` | Human override of AI decision |
| `system.startup` | System initialization |
| `system.shutdown` | System shutdown |
| `system.error` | System error occurred |
| `interaction.gesture_detected` | Human gesture recognized |
| `interaction.voice_command` | Voice command received |
| `interaction.user_present` | Human presence detected in interaction zone |
| `interaction.handover_initiated` | Object handover to/from human started |
| `interaction.handover_completed` | Object handover completed |
| `payload.loaded` | Cargo/payload loaded |
| `payload.unloaded` | Cargo/payload unloaded |
| `payload.shifted` | Payload shift detected |
| `payload.temperature_alert` | Payload temperature out of range |
| `coordination.formation_joined` | Robot joined formation |
| `coordination.formation_left` | Robot left formation |
| `coordination.resource_requested` | Shared resource requested |
| `coordination.resource_granted` | Shared resource access granted |
| `coordination.negotiation_complete` | Multi-agent negotiation completed |
| `manipulation.grasp_initiated` | Grasp attempt started |
| `manipulation.grasp_success` | Object successfully grasped |
| `manipulation.grasp_failed` | Grasp attempt failed |
| `manipulation.object_placed` | Object placement completed |
| `manipulation.tool_changed` | End-effector/tool change completed |
| `manipulation.hand_guiding_started` | Hand guiding mode initiated |
| `manipulation.hand_guiding_completed` | Hand guiding mode ended |
| `environment.door_opened` | Door/gate opened |
| `environment.elevator_called` | Elevator requested |
| `environment.elevator_entered` | Entered elevator |
| `custom.*` | Vendor/application-specific events |

---

## Source Types

| Source Type | Description | Examples |
|-------------|-------------|----------|
| `amr` | Autonomous Mobile Robot | Warehouse AMRs, logistics robots |
| `agv` | Automated Guided Vehicle | Factory AGVs, line-following robots |
| `cobot` | Collaborative Robot | Industrial arms, assembly cobots |
| `drone` | Unmanned Aerial Vehicle | Delivery drones, inspection drones |
| `delivery_robot` | Last-mile delivery robot | Sidewalk delivery, campus delivery |
| `av` | Autonomous Vehicle | Robotaxis, autonomous trucks |
| `inspection_robot` | Inspection/patrol robot | Security, infrastructure inspection |
| `agricultural_robot` | Agricultural autonomous system | Harvesters, sprayers, monitoring |
| `construction_robot` | Construction autonomous system | Excavators, concrete printers |
| `medical_robot` | Medical/healthcare robot | Surgical, logistics, disinfection |
| `humanoid` | Humanoid robot | General-purpose humanoids |
| `legged_robot` | Legged locomotion robot | Quadrupeds, bipeds |
| `marine_robot` | Marine autonomous system | AUVs, surface vessels |
| `simulation` | Simulated robot | Digital twins, test environments |
| `edge_gateway` | Edge computing node | On-prem gateways, edge compute nodes |
| `fleet_manager` | Fleet management system | Orchestration, dispatch |
| `external_sensor` | External sensor system | Cameras, LiDAR not on robot |
| `human` | Human actor | Workers, pedestrians (for context) |
| `swarm_robot` | Swarm/collective robot | Coordinated micro-robots, swarm systems |
| `exoskeleton` | Powered exoskeleton | Industrial exoskeletons, medical assist devices |
| `telepresence_robot` | Telepresence/remote presence robot | Remote collaboration, telemedicine robots |
| `cleaning_robot` | Commercial cleaning robot | Floor scrubbers, vacuum robots, disinfection |
| `hospitality_robot` | Hospitality/service robot | Hotel delivery, restaurant service, concierge |
| `retail_robot` | Retail automation robot | Inventory scanning, shelf stocking, customer assist |
| `mining_robot` | Mining autonomous system | Underground/surface mining, tunnel boring |
| `forestry_robot` | Forestry autonomous system | Tree planting, logging, forest monitoring |
| `space_robot` | Space autonomous system | Orbital servicing, planetary rovers |
| `underwater_robot` | Underwater robot (ROV/AUV) | Subsea inspection, research, salvage |
| `custom.*` | Vendor-defined source type | Platform-specific |

---

## Domain Schemas

### 1. Identity Domain

Identifies the source and its organizational context.

```json
{
  "identity": {
    "source_id": "robot-001",
    "source_type": "amr",
    "source_name": "Picker Alpha",
    "platform": "locus_robotics",
    "platform_version": "3.2.1",
    "firmware_version": "2024.12.01",
    "hardware_revision": "rev-c",
    "serial_number": "LR-2024-00001234",
    "fleet_id": "warehouse-east-fleet-1",
    "site_id": "site-chicago-dc1",
    "zone_id": "zone-aisle-7",
    "organization_id": "org-acme-corp",
    "tags": ["picker", "high-priority", "shift-1"]
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `source_id` | string | Unique source identifier (required) |
| `source_type` | string | Source type classification (required) |
| `source_name` | string | Human-readable name |
| `platform` | string | Platform/manufacturer identifier |
| `platform_version` | string | Platform software version |
| `firmware_version` | string | Firmware version |
| `hardware_revision` | string | Hardware revision |
| `serial_number` | string | Manufacturer serial number |
| `fleet_id` | string | Fleet grouping identifier |
| `site_id` | string | Physical site/facility identifier |
| `zone_id` | string | Current operational zone |
| `organization_id` | string | Customer/tenant organization |
| `tags` | array[string] | Arbitrary classification tags |

---

### 2. Location Domain

Physical location and coordinate systems.

```json
{
  "location": {
    "coordinate_system": "wgs84",
    "latitude": 41.8781,
    "longitude": -87.6298,
    "altitude_m": 182.5,
    "altitude_reference": "msl",
    "horizontal_accuracy_m": 0.5,
    "vertical_accuracy_m": 1.0,
    "heading_deg": 45.0,
    "heading_reference": "true_north",
    "floor": 2,
    "building": "warehouse-a",
    "local": {
      "coordinate_frame": "map",
      "x_m": 15.234,
      "y_m": 8.567,
      "z_m": 0.0,
      "roll_deg": 0.0,
      "pitch_deg": 0.0,
      "yaw_deg": 45.0
    },
    "grid": {
      "grid_id": "warehouse-grid-1",
      "cell_x": 15,
      "cell_y": 8,
      "cell_size_m": 1.0
    },
    "semantic": {
      "area": "picking-zone-a",
      "aisle": "A7",
      "position": "rack-15-slot-3"
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `coordinate_system` | string | Coordinate system: `wgs84`, `utm`, `local`, `grid` |
| `latitude` | float | WGS84 latitude (degrees) |
| `longitude` | float | WGS84 longitude (degrees) |
| `altitude_m` | float | Altitude (meters) |
| `altitude_reference` | string | Altitude reference: `msl`, `agl`, `ellipsoid` |
| `horizontal_accuracy_m` | float | Horizontal position accuracy (meters) |
| `vertical_accuracy_m` | float | Vertical position accuracy (meters) |
| `heading_deg` | float | Heading (degrees, 0-360) |
| `heading_reference` | string | Heading reference: `true_north`, `magnetic_north`, `grid_north` |
| `floor` | integer | Building floor number |
| `building` | string | Building identifier |
| `local` | object | Local coordinate frame position |
| `local.coordinate_frame` | string | Frame ID (e.g., `map`, `odom`, `base_link`) |
| `local.x_m`, `y_m`, `z_m` | float | Position in local frame (meters) |
| `local.roll_deg`, `pitch_deg`, `yaw_deg` | float | Orientation in local frame (degrees) |
| `grid` | object | Grid-based position |
| `semantic` | object | Semantic location labels |

---

### 3. Motion Domain

Velocity, acceleration, and movement state.

```json
{
  "motion": {
    "linear_velocity": {
      "x_mps": 1.2,
      "y_mps": 0.0,
      "z_mps": 0.0,
      "speed_mps": 1.2
    },
    "angular_velocity": {
      "roll_dps": 0.0,
      "pitch_dps": 0.0,
      "yaw_dps": 5.5
    },
    "linear_acceleration": {
      "x_mps2": 0.1,
      "y_mps2": 0.0,
      "z_mps2": 9.81
    },
    "angular_acceleration": {
      "roll_dps2": 0.0,
      "pitch_dps2": 0.0,
      "yaw_dps2": 1.0
    },
    "odometry": {
      "distance_traveled_m": 1523.45,
      "distance_session_m": 234.56
    },
    "motion_state": "moving_forward",
    "commanded_velocity": {
      "linear_mps": 1.5,
      "angular_dps": 0.0
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `linear_velocity` | object | Linear velocity components |
| `linear_velocity.speed_mps` | float | Scalar speed (m/s) |
| `angular_velocity` | object | Angular velocity (deg/s) |
| `linear_acceleration` | object | Linear acceleration (m/s²) |
| `angular_acceleration` | object | Angular acceleration (deg/s²) |
| `odometry.distance_traveled_m` | float | Total distance since reset (m) |
| `odometry.distance_session_m` | float | Distance this session (m) |
| `motion_state` | string | Motion classification (see Motion States) |
| `commanded_velocity` | object | Commanded/target velocity |

**Motion States:** `stationary`, `moving_forward`, `moving_backward`, `turning_left`, `turning_right`, `rotating_in_place`, `ascending`, `descending`, `hovering`, `docking`, `undocking`

---

### 4. Power Domain

Battery, charging, and energy management.

```json
{
  "power": {
    "battery": {
      "state_of_charge_pct": 78.5,
      "state_of_health_pct": 95.0,
      "voltage_v": 48.2,
      "current_a": -12.5,
      "power_w": -602.5,
      "temperature_c": 32.0,
      "time_to_empty_min": 95,
      "time_to_full_min": null,
      "cycle_count": 342,
      "chemistry": "lifepo4",
      "capacity_ah": 100.0,
      "capacity_remaining_ah": 78.5
    },
    "charging": {
      "is_charging": false,
      "is_plugged_in": false,
      "charger_id": null,
      "charging_power_w": 0,
      "charging_mode": null
    },
    "power_state": "discharging",
    "power_consumption_w": 602.5,
    "power_budget_pct": 45.0,
    "energy_regenerated_wh": 12.3
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `battery.state_of_charge_pct` | float | Current charge level (0-100%) |
| `battery.state_of_health_pct` | float | Battery health (0-100%) |
| `battery.voltage_v` | float | Battery voltage (V) |
| `battery.current_a` | float | Current draw (A, negative = discharging) |
| `battery.temperature_c` | float | Battery temperature (°C) |
| `battery.time_to_empty_min` | integer | Estimated time to empty (minutes) |
| `battery.chemistry` | string | Battery chemistry type |
| `charging.is_charging` | boolean | Currently charging |
| `charging.charger_id` | string | Connected charger identifier |
| `power_state` | string | Power state classification |
| `power_consumption_w` | float | Current power consumption (W) |

**Power States:** `discharging`, `charging`, `charged`, `hibernating`, `emergency_power`, `external_power`

---

### 5. Operational State Domain

High-level operational mode and task status.

```json
{
  "operational": {
    "mode": "autonomous",
    "state": "executing_task",
    "sub_state": "navigating_to_goal",
    "availability": "available",
    "enabled": true,
    "e_stop_active": false,
    "safety_mode": "normal",
    "uptime_sec": 28800,
    "task": {
      "task_id": "task-2026-0001234",
      "task_type": "pick_and_place",
      "task_state": "in_progress",
      "task_priority": 5,
      "task_started_at": "2026-01-02T10:30:00Z",
      "task_eta": "2026-01-02T10:35:00Z",
      "task_progress_pct": 65.0,
      "task_source": "wms-integration",
      "task_payload": {
        "pick_location": "A7-15-3",
        "drop_location": "staging-1"
      }
    },
    "queue": {
      "queued_tasks": 3,
      "next_task_id": "task-2026-0001235"
    },
    "errors": [
      {
        "error_code": "NAV-001",
        "error_message": "Path blocked by obstacle",
        "error_severity": "warning",
        "error_timestamp": "2026-01-02T10:32:15Z"
      }
    ]
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `mode` | string | Operational mode (see Modes) |
| `state` | string | Current state (see States) |
| `sub_state` | string | Detailed sub-state |
| `availability` | string | Availability for new tasks |
| `enabled` | boolean | System enabled/disabled |
| `e_stop_active` | boolean | Emergency stop engaged |
| `safety_mode` | string | Current safety mode |
| `uptime_sec` | integer | System uptime (seconds) |
| `task` | object | Current task details |
| `queue` | object | Task queue status |
| `errors` | array | Active errors/warnings |

**Modes:** `autonomous`, `manual`, `teleoperated`, `semi_autonomous`, `collaborative`, `learning`, `maintenance`, `emergency`, `idle`, `standby`, `off`

**States:** `idle`, `executing_task`, `waiting`, `charging`, `error`, `maintenance`, `emergency_stop`, `initializing`, `shutting_down`, `paused`

---

### 6. Navigation Domain

Path planning, localization, and navigation status.

```json
{
  "navigation": {
    "localization": {
      "status": "localized",
      "confidence": 0.98,
      "method": "amcl",
      "map_id": "warehouse-east-v3",
      "pose_covariance": [0.01, 0, 0, 0, 0.01, 0, 0, 0, 0.02]
    },
    "slam": {
      "active": false,
      "mode": "localization_only",
      "map_quality": 0.95,
      "loop_closures": 42,
      "keyframes": 1250,
      "map_size_mb": 125.5
    },
    "path": {
      "has_path": true,
      "path_length_m": 15.7,
      "path_eta_sec": 12.5,
      "waypoints_remaining": 3,
      "path_blocked": false,
      "replanning": false
    },
    "goal": {
      "goal_id": "goal-001",
      "goal_x_m": 25.0,
      "goal_y_m": 12.0,
      "goal_yaw_deg": 90.0,
      "distance_to_goal_m": 8.3,
      "goal_type": "pickup"
    },
    "obstacles": {
      "nearest_obstacle_m": 2.5,
      "obstacle_count": 3,
      "dynamic_obstacles": 1,
      "static_obstacles": 2
    },
    "costmap": {
      "inflation_radius_m": 0.5,
      "robot_radius_m": 0.3
    },
    "multi_floor": {
      "current_floor": 1,
      "target_floor": 2,
      "floor_transition_method": "elevator",
      "floor_transition_status": "awaiting_elevator",
      "floors_in_mission": [1, 2]
    },
    "semantic_map": {
      "current_region": "warehouse-zone-a",
      "region_type": "storage",
      "nearby_pois": [
        {"poi_id": "charger-01", "type": "charging_station", "distance_m": 5.2},
        {"poi_id": "door-A3", "type": "door", "distance_m": 8.1}
      ],
      "lane_id": "lane-north-7",
      "allowed_regions": ["zone-a", "zone-b", "corridor-1"],
      "restricted_regions": ["zone-c", "maintenance-area"]
    },
    "planner": {
      "global_planner": "navfn",
      "local_planner": "dwb",
      "behavior_tree": "navigate_to_pose.xml",
      "recovery_active": false,
      "recovery_behavior": null
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `localization.status` | string | Status: `not_localized`, `localizing`, `localized`, `lost` |
| `localization.method` | string | Method: `amcl`, `slam`, `gps`, `uwb`, `fusion` |
| `slam` | object | SLAM system status when active |
| `slam.mode` | string | Mode: `mapping`, `localization_only`, `mapping_and_localization` |
| `path` | object | Current path status |
| `goal` | object | Active navigation goal |
| `obstacles` | object | Detected obstacles summary |
| `costmap` | object | Costmap configuration |
| `multi_floor` | object | Multi-floor navigation status |
| `semantic_map` | object | Semantic/topological map context |
| `planner` | object | Active planner configuration |

---

### 7. Perception Domain

Sensor readings and environmental perception.

```json
{
  "perception": {
    "lidar": [
      {
        "sensor_id": "lidar_front",
        "status": "ok",
        "min_range_m": 0.1,
        "max_range_m": 30.0,
        "scan_rate_hz": 15.0,
        "points_per_scan": 1800,
        "nearest_point_m": 2.3,
        "fov_deg": 270.0
      }
    ],
    "cameras": [
      {
        "sensor_id": "camera_front",
        "status": "ok",
        "resolution": "1920x1080",
        "fps": 30,
        "exposure_mode": "auto",
        "type": "rgb"
      }
    ],
    "depth_sensors": [
      {
        "sensor_id": "depth_front",
        "status": "ok",
        "min_range_m": 0.3,
        "max_range_m": 10.0,
        "type": "stereo"
      }
    ],
    "thermal_cameras": [
      {
        "sensor_id": "thermal_front",
        "status": "ok",
        "resolution": "640x480",
        "fps": 9,
        "min_temp_c": -20.0,
        "max_temp_c": 150.0,
        "hotspot_detected": false,
        "hotspot_temp_c": null
      }
    ],
    "event_cameras": [
      {
        "sensor_id": "event_cam_front",
        "status": "ok",
        "resolution": "1280x720",
        "event_rate_keps": 150.0,
        "dynamic_range_db": 120
      }
    ],
    "tactile_sensors": [
      {
        "sensor_id": "tactile_gripper",
        "status": "ok",
        "contact_detected": true,
        "pressure_kpa": 25.0,
        "slip_detected": false,
        "texture_class": "smooth"
      }
    ],
    "ultrasonic": [
      {
        "sensor_id": "us_rear_left",
        "status": "ok",
        "range_m": 1.2,
        "min_range_m": 0.02,
        "max_range_m": 4.0
      }
    ],
    "radar": [
      {
        "sensor_id": "radar_front",
        "status": "ok",
        "detections": 5,
        "max_range_m": 100.0,
        "type": "fmcw",
        "velocity_measurement": true
      }
    ],
    "imu": {
      "sensor_id": "imu_main",
      "status": "ok",
      "orientation_deg": {"roll": 0.1, "pitch": -0.2, "yaw": 45.3},
      "angular_velocity_dps": {"x": 0.0, "y": 0.0, "z": 5.5},
      "linear_acceleration_mps2": {"x": 0.1, "y": 0.0, "z": 9.81},
      "temperature_c": 28.5
    },
    "gps": {
      "sensor_id": "gps_main",
      "status": "ok",
      "fix_type": "rtk_fixed",
      "satellites_used": 18,
      "hdop": 0.8,
      "vdop": 1.2
    },
    "uwb": [
      {
        "sensor_id": "uwb_main",
        "status": "ok",
        "anchors_detected": 4,
        "position_accuracy_m": 0.1,
        "ranging_mode": "twr"
      }
    ],
    "magnetometer": {
      "sensor_id": "mag_main",
      "status": "ok",
      "heading_deg": 45.2,
      "field_strength_ut": 48.5,
      "calibration_status": "calibrated"
    },
    "barometer": {
      "sensor_id": "baro_main",
      "status": "ok",
      "pressure_hpa": 1013.25,
      "altitude_estimate_m": 185.0
    },
    "wheel_encoders": [
      {
        "sensor_id": "enc_left",
        "status": "ok",
        "ticks": 1234567,
        "velocity_rpm": 120,
        "direction": "forward"
      }
    ],
    "detections": [
      {
        "detection_id": "det-001",
        "object_id": "obj-789",
        "tracking_id": "track-001",
        "class": "pallet",
        "object_type": "pallet",
        "confidence": 0.95,
        "distance_m": 3.2,
        "bearing_deg": 15.0,
        "velocity_mps": 0.0,
        "bounding_box": {"x": 100, "y": 50, "w": 120, "h": 80},
        "dimensions_m": { "length": 1.2, "width": 0.8, "height": 0.15 },
        "pose": {
          "x_m": 4.5,
          "y_m": 2.1,
          "z_m": 0.0,
          "yaw_deg": 90.0
        },
        "frame_id": "map"
      },
      {
        "detection_id": "det-002",
        "tracking_id": "track-002",
        "class": "person",
        "confidence": 0.92,
        "distance_m": 5.1,
        "bearing_deg": -30.0,
        "velocity_mps": 1.1,
        "bounding_box": {"x": 200, "y": 50, "w": 80, "h": 200},
        "pose_estimation": {
          "skeleton_detected": true,
          "body_orientation_deg": 180
        }
      }
    ],
    "semantic_segmentation": {
      "model_id": "segnet-v2",
      "classes_detected": ["floor", "wall", "person", "forklift", "pallet"],
      "drivable_area_pct": 75.0
    },
    "point_cloud": {
      "source": "lidar_front",
      "points_count": 65000,
      "frame_id": "base_link",
      "timestamp": "2026-01-02T10:34:58.123456Z"
    },
    "environment": {
      "temperature_c": 22.0,
      "humidity_pct": 45.0,
      "light_lux": 350,
      "noise_db": 65,
      "air_quality_aqi": 42,
      "co2_ppm": 420,
      "dust_ugm3": 15.0
    }
  }
}
```

---

### 8. Safety Domain

Safety-related status and events.

```json
{
  "safety": {
    "safety_state": "normal",
    "e_stop": {
      "active": false,
      "source": null,
      "timestamp": null
    },
    "safety_zones": {
      "protective_stop_active": false,
      "reduced_speed_active": true,
      "current_zone_type": "human_collaboration",
      "zone_speed_limit_mps": 0.5
    },
    "proximity": {
      "nearest_human_m": 3.2,
      "nearest_robot_m": 8.5,
      "nearest_obstacle_m": 2.3,
      "collision_imminent": false,
      "time_to_collision_sec": null
    },
    "bumpers": {
      "front_triggered": false,
      "rear_triggered": false
    },
    "safety_lidar": {
      "protective_field_clear": true,
      "warning_field_clear": false
    },
    "force_torque": {
      "contact_detected": false,
      "contact_force_n": 0.0,
      "force_limit_n": 150.0
    },
    "violations": [
      {
        "violation_id": "viol-001",
        "rule_id": "speed-limit-zone-a",
        "violation_type": "speed_limit_exceeded",
        "severity": "warning",
        "timestamp": "2026-01-02T10:32:00Z",
        "details": {
          "limit_mps": 0.5,
          "actual_mps": 0.7
        }
      }
    ],
    "safety_score": 95.0,
    "collaborative_operation": {
      "enabled": true,
      "operation_mode": "speed_separation_monitoring",
      "safety_config_checksum": "A7F3B2E9",
      "config_last_verified": "2026-01-12T08:00:00Z",
      "mode_transition_allowed": true,
      "speed_separation": {
        "active": true,
        "min_protective_separation_m": 0.5,
        "current_separation_m": 1.8,
        "separation_violation": false,
        "max_tcp_speed_mps": 0.25,
        "current_tcp_speed_mps": 0.15,
        "max_joint_speed_pct": 50,
        "stopping_time_ms": 150,
        "stopping_distance_m": 0.05,
        "human_approach_speed_mps": 1.6,
        "calculation_method": "iso_ts_15066_annex_a"
      },
      "power_force_limits": {
        "max_static_force_n": 140.0,
        "max_transient_force_n": 150.0,
        "max_pressure_pa": 110000,
        "max_power_w": 80.0,
        "current_force_n": 0.0,
        "current_power_w": 0.0,
        "limit_exceeded": false
      },
      "contact_event": {
        "contact_detected": false,
        "contact_type": null,
        "body_region": null,
        "measured_force_n": 0.0,
        "measured_pressure_pa": 0.0,
        "contact_duration_ms": 0,
        "limit_exceeded": false,
        "timestamp": null
      },
      "body_region_limits": {
        "skull_face": {
          "max_quasi_static_force_n": 130,
          "max_quasi_static_pressure_pa": 110000,
          "max_transient_force_n": 130,
          "max_transient_power_w": 65
        },
        "forehead": {
          "max_quasi_static_force_n": 130,
          "max_quasi_static_pressure_pa": 110000,
          "max_transient_force_n": 130,
          "max_transient_power_w": 65
        },
        "neck": {
          "max_quasi_static_force_n": 140,
          "max_quasi_static_pressure_pa": 110000,
          "max_transient_force_n": 150,
          "max_transient_power_w": 75
        },
        "back_shoulders": {
          "max_quasi_static_force_n": 210,
          "max_quasi_static_pressure_pa": 110000,
          "max_transient_force_n": 210,
          "max_transient_power_w": 105
        },
        "chest": {
          "max_quasi_static_force_n": 140,
          "max_quasi_static_pressure_pa": 110000,
          "max_transient_force_n": 140,
          "max_transient_power_w": 70
        },
        "abdomen_pelvis": {
          "max_quasi_static_force_n": 140,
          "max_quasi_static_pressure_pa": 110000,
          "max_transient_force_n": 140,
          "max_transient_power_w": 70
        },
        "upper_arms_elbows": {
          "max_quasi_static_force_n": 150,
          "max_quasi_static_pressure_pa": 110000,
          "max_transient_force_n": 150,
          "max_transient_power_w": 75
        },
        "forearms_hands": {
          "max_quasi_static_force_n": 140,
          "max_quasi_static_pressure_pa": 110000,
          "max_transient_force_n": 160,
          "max_transient_power_w": 80
        },
        "thighs_knees": {
          "max_quasi_static_force_n": 220,
          "max_quasi_static_pressure_pa": 110000,
          "max_transient_force_n": 220,
          "max_transient_power_w": 110
        },
        "lower_legs": {
          "max_quasi_static_force_n": 220,
          "max_quasi_static_pressure_pa": 110000,
          "max_transient_force_n": 220,
          "max_transient_power_w": 110
        }
      },
      "monitored_stop": {
        "active": false,
        "all_axes_stationary": true,
        "position_monitoring_active": true,
        "max_position_deviation_mm": 0.5,
        "stop_category": "category_2"
      },
      "workspace_monitoring": {
        "collaborative_workspace_active": true,
        "workspace_id": "cobot_cell_01",
        "humans_in_workspace": 1,
        "robot_stopped_for_human": false,
        "workspace_boundaries_m": {
          "x_min": -1.5,
          "x_max": 1.5,
          "y_min": -1.5,
          "y_max": 1.5,
          "z_min": 0.0,
          "z_max": 2.0
        }
      }
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `safety_state` | string | Overall safety status: `normal`, `warning`, `protective_stop`, `emergency_stop`, `fault` |
| `e_stop` | object | Emergency stop status and metadata |
| `safety_zones` | object | Safety zone status for protected/warning zones |
| `proximity` | object | Proximity monitoring to humans, robots, obstacles |
| `bumpers` | object | Physical bumper sensor status |
| `safety_lidar` | object | Safety-rated LiDAR field status |
| `force_torque` | object | Force/torque contact detection |
| `violations` | array | Active safety violations/warnings |
| `safety_score` | float | Overall safety score (0-100) |
| `collaborative_operation` | object | Collaborative robot operation per ANSI/RIA R15.06, RIA TR R15.606, ISO/TS 15066 |
| `collaborative_operation.enabled` | boolean | Collaborative operation capability enabled |
| `collaborative_operation.operation_mode` | string | Current collaborative operation mode (see Collaborative Operation Modes) |
| `collaborative_operation.safety_config_checksum` | string | Unique identifier for safety parameter configuration (RIA TR R15.606 Section 5.4.2) |
| `collaborative_operation.config_last_verified` | string | ISO 8601 timestamp of last safety configuration verification |
| `collaborative_operation.mode_transition_allowed` | boolean | Whether mode transitions are currently allowed |
| `collaborative_operation.speed_separation` | object | Speed and separation monitoring (ANSI/RIA R15.06 Part 1 Section 5.10.4) |
| `collaborative_operation.speed_separation.min_protective_separation_m` | float | Minimum protective separation distance (meters) |
| `collaborative_operation.speed_separation.current_separation_m` | float | Current separation distance to nearest human (meters) |
| `collaborative_operation.speed_separation.max_tcp_speed_mps` | float | Maximum tool center point speed (m/s) |
| `collaborative_operation.speed_separation.stopping_time_ms` | float | Robot stopping time (milliseconds) |
| `collaborative_operation.power_force_limits` | object | Power and force limiting (ANSI/RIA R15.06 Part 1 Section 5.10.5) |
| `collaborative_operation.power_force_limits.max_static_force_n` | float | Maximum quasi-static contact force (Newtons) |
| `collaborative_operation.power_force_limits.max_transient_force_n` | float | Maximum transient contact force (Newtons) |
| `collaborative_operation.power_force_limits.max_pressure_pa` | float | Maximum contact pressure (Pascals) |
| `collaborative_operation.contact_event` | object | Contact event detection and classification |
| `collaborative_operation.contact_event.contact_type` | string | Contact type: `quasi_static` (clamping), `transient` (free body) |
| `collaborative_operation.contact_event.body_region` | string | Body region contacted (see Body Regions) |
| `collaborative_operation.body_region_limits` | object | Per-body-region biomechanical limits per ISO/TS 15066 Annex A Table A.2 |
| `collaborative_operation.monitored_stop` | object | Safety-rated monitored stop (ANSI/RIA R15.06 Part 1 Section 5.10.2) |
| `collaborative_operation.monitored_stop.all_axes_stationary` | boolean | All robot axes confirmed stationary |
| `collaborative_operation.monitored_stop.position_monitoring_active` | boolean | Continuous position monitoring active |
| `collaborative_operation.workspace_monitoring` | object | Collaborative workspace monitoring |
| `collaborative_operation.workspace_monitoring.humans_in_workspace` | integer | Number of humans detected in collaborative workspace |

**Collaborative Operation Modes:** (per ANSI/RIA R15.06-2012 and RIA TR R15.606-2016)
- `safety_rated_monitored_stop`: Robot in verified stopped state, human can enter workspace
- `hand_guiding`: Human manually guides robot via teach pendant or direct contact
- `speed_separation_monitoring`: Robot and human work concurrently with maintained separation and speed limits
- `power_force_limiting`: Robot and human can have physical contact within biomechanical limits

**Body Regions:** (per ISO/TS 15066 Annex A Table A.1)
- `skull_face`: Skull and face (except forehead)
- `forehead`: Forehead region
- `neck`: Neck region (high risk)
- `back_shoulders`: Back and shoulders
- `chest`: Chest region (moderate risk)
- `abdomen_pelvis`: Abdomen and pelvis (moderate risk)
- `upper_arms_elbows`: Upper arms and elbow joints
- `forearms_hands`: Forearms and hands (most common contact)
- `thighs_knees`: Thighs and knee joints
- `lower_legs`: Lower legs and shins

---

### 9. Actuators Domain

Motor, joint, and actuator status.

```json
{
  "actuators": {
    "drive_motors": [
      {
        "motor_id": "motor_left",
        "status": "ok",
        "position_rad": null,
        "velocity_rpm": 120,
        "torque_nm": 5.2,
        "current_a": 8.5,
        "temperature_c": 45.0,
        "duty_cycle_pct": 60.0
      },
      {
        "motor_id": "motor_right",
        "status": "ok",
        "velocity_rpm": 118,
        "torque_nm": 5.1,
        "current_a": 8.3,
        "temperature_c": 44.0,
        "duty_cycle_pct": 58.0
      }
    ],
    "joints": [
      {
        "joint_id": "joint_1",
        "joint_name": "shoulder_pan",
        "joint_type": "revolute",
        "status": "ok",
        "position_rad": 1.57,
        "velocity_radps": 0.0,
        "effort_nm": 0.0,
        "temperature_c": 35.0,
        "position_error_rad": 0.001,
        "min_limit_rad": -3.14,
        "max_limit_rad": 3.14,
        "at_limit": false
      }
    ],
    "grippers": [
      {
        "gripper_id": "gripper_main",
        "gripper_type": "parallel_jaw",
        "status": "ok",
        "state": "closed",
        "position_mm": 0.0,
        "force_n": 25.0,
        "object_detected": true,
        "max_aperture_mm": 85.0,
        "max_force_n": 100.0
      }
    ],
    "suction": [
      {
        "suction_id": "suction_main",
        "status": "ok",
        "vacuum_active": true,
        "vacuum_level_kpa": -80.0,
        "object_detected": true,
        "leak_detected": false,
        "cups": 4,
        "cups_sealed": 4
      }
    ],
    "lifts": [
      {
        "lift_id": "lift_main",
        "lift_type": "scissor",
        "status": "ok",
        "height_m": 0.8,
        "height_pct": 50.0,
        "load_kg": 15.0,
        "max_load_kg": 50.0,
        "max_height_m": 1.6,
        "velocity_mps": 0.0
      }
    ],
    "conveyors": [
      {
        "conveyor_id": "conveyor_top",
        "status": "ok",
        "running": true,
        "speed_mps": 0.3,
        "direction": "forward",
        "load_detected": true
      }
    ],
    "steering": [
      {
        "steering_id": "steering_front",
        "status": "ok",
        "angle_deg": 15.0,
        "target_angle_deg": 15.0,
        "steering_type": "ackermann",
        "max_angle_deg": 35.0
      }
    ],
    "hydraulics": [
      {
        "system_id": "hydraulic_main",
        "status": "ok",
        "pressure_bar": 200.0,
        "max_pressure_bar": 250.0,
        "fluid_level_pct": 95.0,
        "fluid_temperature_c": 55.0,
        "pump_running": true,
        "cylinders": [
          {
            "cylinder_id": "boom_extend",
            "position_pct": 45.0,
            "force_kn": 50.0,
            "extending": false
          }
        ]
      }
    ],
    "pneumatics": [
      {
        "system_id": "pneumatic_main",
        "status": "ok",
        "pressure_bar": 6.0,
        "max_pressure_bar": 8.0,
        "compressor_running": false,
        "air_consumption_lpm": 15.0,
        "valves": [
          {
            "valve_id": "gripper_close",
            "state": "energized",
            "type": "5_2_way"
          }
        ]
      }
    ],
    "propellers": [
      {
        "propeller_id": "prop_front_left",
        "status": "ok",
        "rpm": 5500,
        "thrust_n": 12.5,
        "power_w": 150.0,
        "temperature_c": 45.0
      }
    ],
    "tracks": [
      {
        "track_id": "track_left",
        "status": "ok",
        "velocity_mps": 0.8,
        "tension_ok": true,
        "wear_pct": 15.0
      }
    ],
    "legs": [
      {
        "leg_id": "leg_front_left",
        "status": "ok",
        "ground_contact": true,
        "force_n": 250.0,
        "joints": [
          {"joint": "hip", "angle_deg": 45.0},
          {"joint": "knee", "angle_deg": 90.0},
          {"joint": "ankle", "angle_deg": 15.0}
        ],
        "gait_phase": "stance"
      }
    ],
    "wheels": [
      {
        "wheel_id": "wheel_front_left",
        "status": "ok",
        "velocity_rpm": 60,
        "steering_angle_deg": 0.0,
        "traction": "ok",
        "pressure_bar": 2.4,
        "wear_pct": 10.0
      }
    ]
  }
}
```

---

### 10. Communication Domain

Network, fleet, and integration status.

```json
{
  "communication": {
    "network": {
      "connected": true,
      "connection_type": "wifi",
      "ssid": "warehouse-iot",
      "signal_strength_dbm": -65,
      "ip_address": "192.168.1.101",
      "mac_address": "AA:BB:CC:DD:EE:FF",
      "latency_ms": 15,
      "bandwidth_mbps": 50.0,
      "packet_loss_pct": 0.1
    },
    "cellular": {
      "connected": false,
      "carrier": null,
      "signal_strength_dbm": null,
      "technology": null
    },
    "fleet": {
      "fleet_connected": true,
      "fleet_manager_id": "fleet-mgr-1",
      "last_heartbeat": "2026-01-02T10:34:55Z",
      "assigned_dispatcher": "dispatcher-east"
    },
    "integrations": [
      {
        "integration_id": "wms-sap",
        "status": "connected",
        "last_sync": "2026-01-02T10:34:50Z"
      }
    ],
    "vendor_extensions": {
      "agent_version": "1.2.0",
      "connected": true,
      "buffer_size": 150,
      "buffer_capacity": 10000,
      "last_upload": "2026-01-02T10:34:58Z",
      "upload_rate_eps": 10.0
    }
  }
}
```

---

### 11. Compute Domain

Onboard compute and resource utilization.

```json
{
  "compute": {
    "cpu": {
      "usage_pct": 45.0,
      "temperature_c": 55.0,
      "frequency_mhz": 2400,
      "cores": 8
    },
    "memory": {
      "used_mb": 4096,
      "total_mb": 16384,
      "usage_pct": 25.0
    },
    "gpu": {
      "usage_pct": 30.0,
      "memory_used_mb": 2048,
      "memory_total_mb": 8192,
      "temperature_c": 60.0
    },
    "storage": {
      "used_gb": 50.0,
      "total_gb": 256.0,
      "usage_pct": 19.5
    },
    "processes": {
      "navigation_stack": {"status": "running", "cpu_pct": 15.0, "memory_mb": 512},
      "perception": {"status": "running", "cpu_pct": 20.0, "memory_mb": 1024},
      "planner": {"status": "running", "cpu_pct": 5.0, "memory_mb": 256}
    },
    "ros": {
      "ros_version": "humble",
      "node_count": 25,
      "topic_count": 150,
      "master_connected": true
    }
  }
}
```

---

### 12. AI/Reasoning Domain

AI model decisions, confidence, and explainability.

```json
{
  "ai": {
    "models": [
      {
        "model_id": "nav-planner-v3",
        "model_type": "path_planning",
        "model_version": "3.2.1",
        "status": "active",
        "inference_time_ms": 12.5,
        "last_inference": "2026-01-02T10:34:58Z"
      },
      {
        "model_id": "object-detector-v2",
        "model_type": "perception",
        "model_version": "2.1.0",
        "status": "active",
        "inference_time_ms": 33.0,
        "fps": 30
      }
    ],
    "decisions": [
      {
        "decision_id": "dec-001",
        "timestamp": "2026-01-02T10:34:58Z",
        "model_id": "nav-planner-v3",
        "decision_type": "path_selection",
        "decision": "path_b",
        "confidence": 0.92,
        "alternatives": [
          {"option": "path_a", "score": 0.85},
          {"option": "path_c", "score": 0.78}
        ],
        "factors": [
          {"factor": "distance", "weight": 0.4, "value": 0.9},
          {"factor": "congestion", "weight": 0.3, "value": 0.95},
          {"factor": "battery", "weight": 0.3, "value": 0.88}
        ],
        "human_override": false
      }
    ],
    "anomalies": [
      {
        "anomaly_id": "anom-001",
        "timestamp": "2026-01-02T10:30:00Z",
        "anomaly_type": "behavior_deviation",
        "severity": "low",
        "description": "Unusual dwell time at location",
        "confidence": 0.75
      }
    ]
  }
}
```

---

### 13. Maintenance Domain

Health, diagnostics, and maintenance tracking.

```json
{
  "maintenance": {
    "health_score": 92.0,
    "diagnostics": [
      {
        "component": "motor_left",
        "status": "ok",
        "health_pct": 95.0,
        "last_checked": "2026-01-02T08:00:00Z",
        "predicted_failure": null
      },
      {
        "component": "battery_pack",
        "status": "warning",
        "health_pct": 78.0,
        "last_checked": "2026-01-02T08:00:00Z",
        "predicted_failure": "2026-03-15",
        "recommendation": "Schedule battery replacement"
      }
    ],
    "maintenance_due": [
      {
        "maintenance_type": "wheel_inspection",
        "due_date": "2026-01-15",
        "due_hours": 50,
        "priority": "medium"
      }
    ],
    "maintenance_history": [
      {
        "maintenance_id": "maint-001",
        "maintenance_type": "software_update",
        "performed_at": "2026-01-01T06:00:00Z",
        "performed_by": "tech-042",
        "notes": "Updated navigation stack to v3.2.1"
      }
    ],
    "operating_hours": 2450.5,
    "operating_hours_since_maintenance": 150.5
  }
}
```

---

### 14. Context Domain

External context and environmental factors.

```json
{
  "context": {
    "time": {
      "local_time": "2026-01-02T04:35:00-06:00",
      "timezone": "America/Chicago",
      "shift": "morning",
      "is_peak_hours": true
    },
    "facility": {
      "facility_id": "warehouse-chicago-1",
      "facility_type": "distribution_center",
      "operational_status": "normal",
      "occupancy_pct": 65.0
    },
    "weather": {
      "condition": "clear",
      "temperature_c": -5.0,
      "humidity_pct": 35.0,
      "wind_speed_mps": 3.0
    },
    "traffic": {
      "congestion_level": "medium",
      "robots_in_zone": 5,
      "humans_in_zone": 3
    },
    "events": [
      {
        "event_type": "dock_door_open",
        "event_id": "door-3",
        "timestamp": "2026-01-02T10:30:00Z"
      }
    ]
  }
}
```

---

### 15. Payload/Cargo Domain

Cargo, payload, and load management for logistics and delivery robots.

```json
{
  "payload": {
    "load_status": "loaded",
    "total_weight_kg": 25.5,
    "max_capacity_kg": 50.0,
    "center_of_gravity": {
      "x_m": 0.1,
      "y_m": 0.0,
      "z_m": 0.3
    },
    "compartments": [
      {
        "compartment_id": "main",
        "status": "occupied",
        "weight_kg": 25.5,
        "volume_used_pct": 60.0,
        "door_state": "closed",
        "locked": true,
        "temperature_c": 4.0,
        "temperature_setpoint_c": 4.0,
        "humidity_pct": 45.0
      }
    ],
    "items": [
      {
        "object_id": "obj-456",
        "item_id": "ORD-2026-00789",
        "object_type": "package",
        "object_class": "cardboard_box",
        "weight_kg": 5.2,
        "dimensions_m": { "length": 0.4, "width": 0.3, "height": 0.2 },
        "loaded_at": "2026-01-02T10:15:00Z",
        "destination": "dock-5",
        "fragile": false,
        "temperature_sensitive": true,
        "barcode": "1234567890123",
        "rfid_tag": "RFID-ABC-123",
        "asset_id": "ASSET-00123"
      }
    ],
    "load_shifted": false,
    "overweight": false,
    "secured": true
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `load_status` | string | Load state: `empty`, `loading`, `loaded`, `unloading` |
| `total_weight_kg` | float | Total payload weight (kg) |
| `max_capacity_kg` | float | Maximum payload capacity (kg) |
| `center_of_gravity` | object | Payload center of gravity offset |
| `compartments` | array | Individual storage compartments |
| `compartments[].temperature_c` | float | Compartment temperature for cold chain |
| `items` | array | Individual cargo items/packages (see `object_ref`) |
| `items[].object_id` | string | Session-stable ID linking to perception/manipulation |
| `items[].item_id` | string | External business identifier (WMS order, SKU, asset tag) |
| `items[].object_type` | string | Object type: `package`, `pallet`, `tote`, `bin`, `part`, etc. |
| `items[].object_class` | string | Specific classification from perception |
| `items[].dimensions_m` | object | Physical dimensions |
| `items[].asset_id` | string | Asset management system ID |
| `load_shifted` | boolean | Payload shift detected |
| `secured` | boolean | Payload properly secured |

---

### 16. Manipulation Domain

Robot arm manipulation, grasping, and end-effector state for cobots and manipulators.

```json
{
  "manipulation": {
    "arm_id": "arm_main",
    "arm_state": "moving",
    "control_mode": "position",
    "hand_guiding": {
      "active": false,
      "enabled_device_active": false,
      "max_speed_mps": 0.25,
      "force_feedback_enabled": true,
      "teaching_mode": false,
      "waypoint_recorded": false
    },
    "end_effector": {
      "effector_id": "gripper_2f",
      "effector_type": "parallel_gripper",
      "state": "grasping",
      "aperture_mm": 45.0,
      "force_n": 15.0,
      "object_detected": true,
      "tool_center_point": {
        "x_m": 0.85,
        "y_m": 0.12,
        "z_m": 0.45,
        "roll_deg": 0.0,
        "pitch_deg": 90.0,
        "yaw_deg": 45.0
      }
    },
    "workspace": {
      "in_workspace": true,
      "near_singularity": false,
      "reach_pct": 75.0,
      "collision_imminent": false,
      "restricted_zone_active": false
    },
    "motion_plan": {
      "has_plan": true,
      "plan_id": "plan-001",
      "waypoints_remaining": 5,
      "time_to_complete_sec": 2.5,
      "planner": "ompl_rrt_star"
    },
    "grasp": {
      "grasp_id": "grasp-001",
      "object_id": "obj-123",
      "item_id": "PART-A-001",
      "object_type": "part",
      "object_class": "metal_bracket",
      "tracking_id": "track-001",
      "grasp_type": "pinch",
      "grasp_quality": 0.92,
      "stable": true,
      "slip_detected": false
    },
    "tool_changer": {
      "enabled": true,
      "current_tool": "gripper_2f",
      "available_tools": ["gripper_2f", "suction_cup", "screwdriver"],
      "tool_locked": true
    },
    "force_control": {
      "enabled": true,
      "mode": "impedance",
      "target_force_n": 10.0,
      "measured_force_n": 9.8,
      "contact_detected": true
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `arm_state` | string | Arm state: `idle`, `moving`, `holding`, `collision_stop` |
| `control_mode` | string | Control mode: `position`, `velocity`, `force`, `impedance`, `hand_guiding` |
| `hand_guiding` | object | Hand guiding mode status (ANSI/RIA R15.06 Part 1 Section 5.10.3) |
| `hand_guiding.active` | boolean | Hand guiding currently active |
| `hand_guiding.enabled_device_active` | boolean | Three-position enabling device engaged (if required) |
| `hand_guiding.max_speed_mps` | float | Maximum TCP speed during hand guiding (m/s) |
| `hand_guiding.force_feedback_enabled` | boolean | Force feedback/gravity compensation active |
| `hand_guiding.teaching_mode` | boolean | Teaching/programming mode active |
| `end_effector` | object | Current end-effector status |
| `end_effector.tool_center_point` | object | TCP pose in robot frame |
| `workspace` | object | Workspace and collision status |
| `motion_plan` | object | Current motion plan status |
| `grasp` | object | Current grasp information (uses `object_ref`) |
| `grasp.object_id` | string | Session-stable ID of grasped object |
| `grasp.item_id` | string | External business identifier of grasped object |
| `grasp.tracking_id` | string | Perception tracking ID of grasped object |
| `tool_changer` | object | Automatic tool changer status |
| `force_control` | object | Force/torque control status |

---

### 17. Human-Robot Interaction (HRI) Domain

Human interaction, collaboration, and social robotics data.

```json
{
  "hri": {
    "interaction_state": "collaborative_task",
    "humans_detected": 2,
    "tracked_humans": [
      {
        "human_id": "human-001",
        "tracking_confidence": 0.95,
        "distance_m": 1.5,
        "bearing_deg": 30.0,
        "velocity_mps": 0.5,
        "body_pose": "standing",
        "attention_on_robot": true,
        "gesture_detected": "wave",
        "gesture_confidence": 0.88,
        "face_detected": true,
        "emotion": "neutral",
        "ppe_detected": {
          "safety_vest": true,
          "hard_hat": true,
          "safety_glasses": false
        },
        "zone": "collaboration_zone"
      }
    ],
    "voice": {
      "voice_activity_detected": true,
      "speech_recognized": "pick up the box",
      "speech_confidence": 0.91,
      "speaker_id": "human-001",
      "language": "en-US",
      "command_parsed": {
        "intent": "pick_object",
        "object": "box",
        "confidence": 0.89
      }
    },
    "handover": {
      "in_progress": true,
      "direction": "robot_to_human",
      "object_id": "obj-123",
      "item_id": "TOOL-WR-15",
      "object_type": "tool",
      "object_class": "wrench",
      "human_id": "human-001",
      "phase": "extending",
      "ready_to_release": false
    },
    "social": {
      "greeting_given": true,
      "acknowledgment_pending": false,
      "user_satisfaction_score": 4.2,
      "interaction_duration_sec": 45.0
    },
    "safety_rating": {
      "iso_ts_15066_compliant": true,
      "current_operation_category": "power_and_force_limiting",
      "max_allowed_speed_mps": 0.25,
      "max_allowed_force_n": 140.0
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `interaction_state` | string | Current HRI state: `idle`, `approaching`, `collaborative_task`, `handover` |
| `tracked_humans` | array | Detected and tracked humans in vicinity |
| `tracked_humans[].ppe_detected` | object | Personal Protective Equipment detection |
| `voice` | object | Voice interaction and speech recognition |
| `handover` | object | Object handover state (robot ↔ human, uses `object_ref`) |
| `handover.object_id` | string | Session-stable ID of handover object |
| `handover.item_id` | string | External business identifier of handover object |
| `social` | object | Social interaction metrics (hospitality/service robots) |
| `safety_rating` | object | ISO/TS 15066 collaborative safety status |

---

### 18. Multi-Agent Coordination Domain

Fleet coordination, swarm behavior, and multi-robot collaboration.

```json
{
  "coordination": {
    "fleet_role": "follower",
    "formation": {
      "formation_id": "form-001",
      "formation_type": "line",
      "position_in_formation": 3,
      "formation_size": 5,
      "leader_id": "robot-001",
      "formation_integrity": 0.95,
      "spacing_error_m": 0.1
    },
    "neighbors": [
      {
        "robot_id": "robot-002",
        "distance_m": 2.5,
        "bearing_deg": -45.0,
        "relative_velocity_mps": 0.0,
        "communication_quality": 0.98,
        "last_heartbeat": "2026-01-02T10:34:58Z"
      }
    ],
    "shared_resources": {
      "pending_requests": 1,
      "held_resources": ["charging_station_3"],
      "waiting_for": ["aisle_7_access"]
    },
    "task_allocation": {
      "allocator": "market_based",
      "current_bid": 85.0,
      "competing_robots": 3,
      "allocation_status": "awarded"
    },
    "swarm": {
      "swarm_id": "swarm-001",
      "swarm_size": 50,
      "consensus_value": 0.87,
      "behavior_mode": "disperse",
      "local_density": 3.2,
      "separation_m": 0.5,
      "alignment_deg": 12.0,
      "cohesion_score": 0.91
    },
    "traffic": {
      "lane_id": "lane-A7",
      "traffic_direction": "north",
      "yielding_to": null,
      "right_of_way": true,
      "intersection_id": null,
      "deadlock_detected": false
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `fleet_role` | string | Role in fleet: `leader`, `follower`, `independent` |
| `formation` | object | Formation status for convoy/swarm operations |
| `neighbors` | array | Nearby robots with communication status |
| `shared_resources` | object | Resource allocation and deadlock avoidance |
| `task_allocation` | object | Multi-robot task allocation status |
| `swarm` | object | Swarm robotics behavior parameters |
| `traffic` | object | Multi-robot traffic management |

---

### 19. Simulation/Digital Twin Domain

Simulation parameters and digital twin synchronization.

```json
{
  "simulation": {
    "is_simulated": true,
    "simulator": {
      "name": "gazebo",
      "version": "harmonic",
      "physics_engine": "bullet",
      "real_time_factor": 1.0,
      "step_size_ms": 1.0
    },
    "scenario": {
      "scenario_id": "scen-warehouse-001",
      "scenario_name": "peak_load_test",
      "scenario_version": "2.1",
      "randomization_seed": 42,
      "parameters": {
        "human_density": "high",
        "obstacle_frequency": "medium",
        "lighting_condition": "normal"
      }
    },
    "digital_twin": {
      "paired_physical_id": "robot-001-physical",
      "sync_status": "synchronized",
      "sync_latency_ms": 50,
      "divergence_detected": false,
      "last_sync": "2026-01-02T10:34:58Z"
    },
    "fidelity": {
      "sensor_noise_enabled": true,
      "physics_fidelity": "high",
      "perception_fidelity": "medium",
      "communication_delay_simulated": true
    },
    "test_oracle": {
      "expected_behavior": "reach_goal",
      "pass_criteria": {
        "max_time_sec": 60,
        "min_safety_score": 90
      },
      "assertions_passed": 12,
      "assertions_failed": 0
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `is_simulated` | boolean | True if data from simulation |
| `simulator` | object | Simulator identification and configuration |
| `scenario` | object | Test scenario information |
| `digital_twin` | object | Physical/digital twin pairing status |
| `fidelity` | object | Simulation fidelity settings |
| `test_oracle` | object | Automated test validation status |

---

### 20. Thermal Management Domain

Temperature monitoring and thermal control systems.

```json
{
  "thermal": {
    "thermal_state": "normal",
    "ambient_temperature_c": 25.0,
    "components": [
      {
        "component_id": "cpu_main",
        "temperature_c": 55.0,
        "max_temperature_c": 95.0,
        "throttling": false,
        "fan_speed_rpm": 2400
      },
      {
        "component_id": "motor_left",
        "temperature_c": 48.0,
        "max_temperature_c": 80.0,
        "throttling": false
      },
      {
        "component_id": "battery_pack",
        "temperature_c": 32.0,
        "max_temperature_c": 45.0,
        "min_temperature_c": 5.0,
        "heating_active": false,
        "cooling_active": false
      }
    ],
    "cooling_system": {
      "type": "active_air",
      "status": "running",
      "fan_count": 2,
      "total_airflow_cfm": 50.0,
      "coolant_temperature_c": null,
      "coolant_flow_lpm": null
    },
    "heating_system": {
      "enabled": true,
      "active": false,
      "power_w": 0
    },
    "enclosure": {
      "internal_temperature_c": 35.0,
      "ingress_protection": "IP65",
      "sealed": true
    },
    "thermal_limits": {
      "operating_min_c": -10.0,
      "operating_max_c": 50.0,
      "storage_min_c": -20.0,
      "storage_max_c": 60.0
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `thermal_state` | string | Overall state: `cold`, `normal`, `warm`, `hot`, `critical` |
| `components` | array | Per-component temperature status |
| `components[].throttling` | boolean | Performance throttling due to temperature |
| `cooling_system` | object | Active cooling status |
| `heating_system` | object | Cold-weather heating status |
| `enclosure` | object | Enclosure environmental protection |
| `thermal_limits` | object | Operating temperature specifications |

---

### 21. Audio Domain

Audio sensing, sound detection, and acoustic monitoring.

```json
{
  "audio": {
    "microphones": [
      {
        "mic_id": "mic_front",
        "status": "ok",
        "gain_db": 0.0,
        "sample_rate_hz": 48000,
        "channels": 4,
        "type": "array"
      }
    ],
    "speakers": [
      {
        "speaker_id": "speaker_main",
        "status": "ok",
        "volume_pct": 70.0,
        "playing": false,
        "current_audio": null
      }
    ],
    "sound_detection": {
      "ambient_level_db": 55.0,
      "peak_level_db": 72.0,
      "events": [
        {
          "event_id": "snd-001",
          "timestamp": "2026-01-02T10:34:55Z",
          "type": "human_speech",
          "confidence": 0.88,
          "direction_deg": 45.0,
          "distance_estimate_m": 3.0,
          "duration_ms": 1500
        },
        {
          "event_id": "snd-002",
          "timestamp": "2026-01-02T10:34:57Z",
          "type": "machinery",
          "confidence": 0.95,
          "direction_deg": -90.0
        }
      ]
    },
    "acoustic_signature": {
      "self_noise_profile": "normal",
      "motor_noise_db": 45.0,
      "anomaly_detected": false,
      "anomaly_type": null
    },
    "alerts": {
      "alarm_detected": false,
      "alarm_type": null,
      "emergency_siren": false,
      "horn_honk": false
    },
    "voice_output": {
      "tts_enabled": true,
      "current_utterance": null,
      "language": "en-US",
      "voice_profile": "professional"
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `microphones` | array | Microphone hardware status |
| `speakers` | array | Speaker hardware status |
| `sound_detection` | object | Detected acoustic events |
| `sound_detection.events[].type` | string | Sound classification: `human_speech`, `machinery`, `alarm`, `impact`, `glass_break` |
| `acoustic_signature` | object | Robot's own noise profile and anomaly detection |
| `alerts` | object | Safety-relevant sound detections |
| `voice_output` | object | Text-to-speech status |

---

### 22. Environment Interaction Domain

Physical environment interactions including doors, elevators, and infrastructure.

```json
{
  "environment_interaction": {
    "doors": [
      {
        "door_id": "door-A1",
        "type": "automatic_sliding",
        "state": "open",
        "controlled_by_robot": true,
        "request_pending": false,
        "access_granted": true,
        "authentication_method": "ble_beacon"
      }
    ],
    "elevators": {
      "current_elevator_id": null,
      "elevator_state": "not_in_elevator",
      "floor_requested": null,
      "current_floor": 1,
      "elevator_summoned": false,
      "doors_open": false,
      "can_enter": false,
      "integration_type": "api"
    },
    "charging_stations": [
      {
        "station_id": "charger-01",
        "distance_m": 15.0,
        "available": true,
        "compatible": true,
        "reserved": false,
        "connector_type": "wireless",
        "max_power_kw": 5.0
      }
    ],
    "infrastructure": {
      "wifi_access_points": 3,
      "ble_beacons_detected": 8,
      "uwb_anchors_detected": 4,
      "traffic_lights": [
        {
          "light_id": "tl-01",
          "state": "green",
          "time_to_change_sec": 15
        }
      ]
    },
    "surface": {
      "surface_type": "concrete",
      "condition": "dry",
      "slope_deg": 0.0,
      "friction_coefficient": 0.7,
      "bump_detected": false,
      "edge_detected": false
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `doors` | array | Door/gate interaction status |
| `elevators` | object | Elevator interaction and floor navigation |
| `charging_stations` | array | Nearby charging infrastructure |
| `infrastructure` | object | Environmental infrastructure detection |
| `surface` | object | Ground surface characteristics |

---

### 23. Compliance/Certification Domain

Regulatory compliance status and certification tracking.

```json
{
  "compliance": {
    "certifications": [
      {
        "standard": "ISO_13482",
        "version": "2014",
        "status": "certified",
        "certificate_id": "CERT-2025-12345",
        "issued_date": "2025-06-15",
        "expiry_date": "2028-06-15",
        "certifying_body": "TÜV SÜD"
      },
      {
        "standard": "ISO_10218",
        "version": "2011",
        "status": "certified",
        "certificate_id": "CERT-2025-12346"
      },
      {
        "standard": "CE_MARKING",
        "status": "compliant",
        "declaration_id": "DOC-2025-001"
      }
    ],
    "functional_safety": {
      "safety_integrity_level": "SIL2",
      "performance_level": "PLd",
      "safety_controller_status": "ok",
      "safety_plc_firmware": "3.1.0",
      "watchdog_status": "ok",
      "last_safety_test": "2026-01-01T06:00:00Z"
    },
    "cybersecurity": {
      "encryption_enabled": true,
      "firmware_signed": true,
      "last_security_scan": "2026-01-01T00:00:00Z",
      "vulnerabilities_known": 0,
      "security_patch_level": "2025-12",
      "authentication_method": "mtls",
      "secure_boot_enabled": true
    },
    "data_privacy": {
      "gdpr_compliant": true,
      "data_retention_days": 90,
      "anonymization_enabled": true,
      "consent_status": "obtained",
      "pii_detected_in_session": false
    },
    "operational_compliance": {
      "operating_license_valid": true,
      "geofence_compliant": true,
      "speed_limit_compliant": true,
      "noise_limit_compliant": true,
      "emissions_compliant": true
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `certifications` | array | Active certifications and standards compliance |
| `functional_safety` | object | IEC 61508/62443 functional safety status |
| `cybersecurity` | object | Cybersecurity posture and compliance |
| `data_privacy` | object | GDPR/privacy compliance status |
| `operational_compliance` | object | Runtime operational rule compliance |

---

### 24. Extensions Domain

Vendor-specific and custom data.

```json
{
  "extensions": {
    "vendor": {
      "vendor_id": "locus_robotics",
      "vendor_schema_version": "2.1.0",
      "data": {
        "locus_task_priority_score": 85,
        "locus_picker_affinity": "zone-a",
        "locus_custom_metric": 123.45
      }
    },
    "platform": {
      "platform_id": "ros2_humble",
      "data": {
        "tf_buffer_size": 1000,
        "costmap_update_frequency": 5.0,
        "custom_param": "value"
      }
    },
    "customer": {
      "customer_id": "acme-corp",
      "data": {
        "shift_assignment": "A",
        "cost_center": "logistics-east",
        "custom_tag": "priority-picker"
      }
    },
    "raw": {
      "format": "ros2_odom",
      "encoding": "json",
      "data": {
        "header": {"stamp": {"sec": 1735820100, "nanosec": 500000000}},
        "pose": {"pose": {"position": {"x": 15.234, "y": 8.567, "z": 0}}},
        "twist": {"twist": {"linear": {"x": 1.2}, "angular": {"z": 0.05}}}
      }
    }
  }
}
```

| Namespace | Purpose |
|-----------|---------|
| `extensions.vendor` | Manufacturer/platform-specific data |
| `extensions.platform` | Middleware/framework-specific data (ROS, custom) |
| `extensions.customer` | Customer/tenant-specific fields |
| `extensions.raw` | Preserved original message (for audit/debug) |
| `extensions.custom.*` | Arbitrary custom namespaces |

---

## Complete UDM Event Example

```json
{
  "udm_version": "0.0.2",
  "event_id": "01941a2b-3c4d-7e8f-9a0b-1c2d3e4f5a6b",
  "event_type": "telemetry.periodic",
  "source_id": "robot-001",
  "source_type": "amr",
  "captured_at": "2026-01-02T10:35:00.123456Z",
  "received_at": "2026-01-02T10:35:00.234567Z",
  "sequence_num": 12345,
  "session_id": "session-2026-01-02-001",
  
  "identity": {
    "source_id": "robot-001",
    "source_type": "amr",
    "source_name": "Picker Alpha",
    "platform": "locus_robotics",
    "fleet_id": "warehouse-east-fleet-1",
    "site_id": "site-chicago-dc1",
    "organization_id": "org-acme-corp"
  },
  
  "location": {
    "coordinate_system": "wgs84",
    "latitude": 41.8781,
    "longitude": -87.6298,
    "heading_deg": 45.0,
    "local": {
      "x_m": 15.234,
      "y_m": 8.567,
      "yaw_deg": 45.0
    }
  },
  
  "motion": {
    "linear_velocity": {
      "speed_mps": 1.2
    },
    "motion_state": "moving_forward"
  },
  
  "power": {
    "battery": {
      "state_of_charge_pct": 78.5,
      "voltage_v": 48.2
    },
    "power_state": "discharging"
  },
  
  "operational": {
    "mode": "autonomous",
    "state": "executing_task",
    "e_stop_active": false,
    "task": {
      "task_id": "task-2026-0001234",
      "task_type": "pick_and_place",
      "task_progress_pct": 65.0
    }
  },
  
  "safety": {
    "safety_state": "normal",
    "proximity": {
      "nearest_human_m": 3.2
    }
  },
  
  "perception": {
    "lidar": [
      {"sensor_id": "lidar_front", "status": "ok"}
    ]
  },
  
  "extensions": {
    "vendor": {
      "vendor_id": "locus_robotics",
      "data": {
        "locus_task_priority_score": 85
      }
    }
  }
}
```

---

## OpenTelemetry Compatibility Layer

Conforming implementations MAY provide an OpenTelemetry exporter that maps UDM events to OTel primitives as follows:

| UDM Domain | OTel Primitive | Mapping |
|------------|----------------|---------|
| `event_type: telemetry.*` | OTel Metrics | Gauge/Counter for numeric fields |
| `event_type: task.*` | OTel Spans | Span per task lifecycle |
| `event_type: state.*` | OTel Logs | Structured log event |
| `event_type: safety.*` | OTel Logs + Metrics | Log event + violation counter |
| `identity.*` | OTel Resource Attributes | Standard resource identification |
| `location.*` | OTel Attributes | `geo.lat`, `geo.lon`, custom attrs |

The UDM remains the canonical format for any conforming backend; the OTel exporter is an interoperability layer.

---

## Provenance Metadata

For immutability and chain-of-custody requirements, events MAY include provenance metadata:

```json
{
  "provenance": {
    "capture_source": "udm_agent",
    "capture_version": "1.2.0",
    "capture_host": "robot-001",
    "ingest_node": "udm-ingest-east-1",
    "ingest_timestamp": "2026-01-02T10:35:00.345678Z",
    "hash": "sha256:abc123...",
    "previous_hash": "sha256:def456...",
    "signature": "ed25519:...",
    "chain_id": "chain-robot-001-2026-01-02"
  }
}
```

---

## Schema Evolution Guidelines

1. **Adding optional fields:** Allowed in patch versions
2. **Adding new domains:** Allowed in minor versions
3. **Deprecating fields:** Mark with `deprecated: true`, remove in next major version
4. **Renaming fields:** Requires major version; provide migration guide
5. **Changing types:** Requires major version
6. **Extensions:** Vendor extensions can evolve independently of core schema

---

## Data Quality & Validation

### Field Validation Rules

| Field Type | Validation |
|------------|------------|
| `event_id` | UUID format (v4 or v7), globally unique |
| `captured_at` | ISO 8601, must not be in future by >1 second |
| `source_id` | Non-empty string, max 256 characters |
| `*_pct` | Numeric, 0-100 range |
| `*_deg` (heading) | Numeric, 0-360 range |
| `*_deg` (orientation) | Numeric, -180 to 180 range |
| `latitude` | Numeric, -90 to 90 |
| `longitude` | Numeric, -180 to 180 |
| `*_m` (distance) | Numeric, >= 0 |
| `*_mps` (speed) | Numeric |
| `status` fields | Enum: `ok`, `warning`, `error`, `degraded`, `offline` |

### Data Quality Indicators

Events may include quality metadata:

```json
{
  "data_quality": {
    "completeness_score": 0.95,
    "fields_missing": ["gps.altitude_m"],
    "sensor_health": {
      "lidar_front": "ok",
      "camera_front": "degraded",
      "imu_main": "ok"
    },
    "clock_synchronized": true,
    "clock_offset_ms": 2.5,
    "transmission_latency_ms": 45.0,
    "duplicate_detection": false,
    "out_of_order_sequence": false
  }
}
```

### Null Handling

- Null values indicate "not available" or "not applicable"
- Empty arrays indicate "no items" (distinct from null = "not reported")
- Missing fields inherit defaults (typically null or empty)

---

## Streaming & Batching

### Event Batching

For high-frequency telemetry, events may be batched:

```json
{
  "batch": {
    "batch_id": "batch-001",
    "batch_size": 100,
    "first_event_at": "2026-01-02T10:34:58.000000Z",
    "last_event_at": "2026-01-02T10:34:58.990000Z",
    "compression": "gzip",
    "events": [...]
  }
}
```

### Delta Encoding

For bandwidth optimization, events may report only changed fields:

```json
{
  "delta": {
    "base_event_id": "prev-event-id",
    "changed_fields": {
      "location.local.x_m": 15.5,
      "motion.linear_velocity.speed_mps": 1.3
    }
  }
}
```

---

## Implementation Notes

- All timestamps: ISO 8601 with microsecond precision, UTC preferred
- All distances: meters (m)
- All velocities: meters per second (m/s)
- All angles: degrees (deg), unless radians explicitly noted
- All temperatures: Celsius (°C)
- All currents: Amperes (A)
- All voltages: Volts (V)
- All forces: Newtons (N)
- All torques: Newton-meters (N·m)
- Percentages: 0-100 scale (not 0-1)

---

## Appendix A: Source Type → Domain Relevance Matrix

| Source Type | Identity | Location | Motion | Power | Operational | Navigation | Perception | Safety | Actuators | Communication | Compute | AI | Maintenance | Payload | Manipulation | HRI | Coordination | Simulation | Thermal | Audio | Environment | Compliance |
|-------------|:--------:|:--------:|:------:|:-----:|:-----------:|:----------:|:----------:|:------:|:---------:|:-------------:|:-------:|:--:|:-----------:|:-------:|:------------:|:---:|:------------:|:----------:|:-------:|:-----:|:-----------:|:----------:|
| AMR | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ○ | ✓ | ✓ | ○ | ○ | ✓ | ○ | ○ | ○ | ✓ | ✓ |
| Cobot | ✓ | ○ | ○ | ✓ | ✓ | ○ | ✓ | ✓ | ✓ | ✓ | ✓ | ○ | ✓ | ○ | ✓ | ✓ | ○ | ○ | ✓ | ○ | ○ | ✓ |
| Drone | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ○ | ✓ | ✓ | ○ | ○ | ✓ | ○ | ✓ | ○ | ○ | ✓ |
| AV | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ○ | ✓ | ✓ | ○ | ✓ | ✓ | ✓ | ✓ |
| Delivery Robot | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ○ | ✓ | ✓ | ○ | ✓ | ✓ | ○ | ✓ | ✓ | ✓ | ✓ |
| Humanoid | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ○ | ○ | ✓ | ✓ | ✓ | ✓ |
| Legged Robot | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ○ | ✓ | ○ | ○ | ○ | ○ | ○ | ✓ | ○ | ✓ | ✓ |
| Hospitality Robot | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ○ | ✓ | ✓ | ○ | ✓ | ○ | ○ | ○ | ✓ | ✓ | ✓ |
| Cleaning Robot | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ○ | ✓ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ✓ | ✓ |
| Agricultural Robot | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ○ | ✓ | ✓ | ✓ | ○ | ○ | ○ | ✓ | ○ | ○ | ✓ |
| Construction Robot | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ○ | ✓ | ✓ | ✓ | ○ | ✓ | ○ | ✓ | ○ | ○ | ✓ |
| Mining Robot | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ○ | ✓ | ✓ | ○ | ○ | ✓ | ○ | ✓ | ○ | ○ | ✓ |
| Marine Robot | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ○ | ✓ | ○ | ✓ | ○ | ○ | ○ | ✓ | ✓ | ○ | ✓ |
| Swarm Robot | ✓ | ✓ | ✓ | ✓ | ✓ | ○ | ○ | ✓ | ○ | ✓ | ○ | ○ | ○ | ○ | ○ | ○ | ✓ | ○ | ○ | ○ | ○ | ○ |
| Exoskeleton | ✓ | ○ | ✓ | ✓ | ✓ | ○ | ○ | ✓ | ✓ | ✓ | ✓ | ○ | ✓ | ○ | ○ | ✓ | ○ | ○ | ✓ | ○ | ○ | ✓ |
| Medical Robot | ✓ | ○ | ○ | ✓ | ✓ | ○ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ○ | ✓ | ✓ | ○ | ○ | ✓ | ○ | ○ | ✓ |
| Simulation | ✓ | ✓ | ✓ | ○ | ✓ | ✓ | ○ | ✓ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ✓ | ○ | ○ | ○ | ○ |

✓ = Typically populated | ○ = Optional/Varies

---

## Appendix B: Vendor Extension Registry

To ensure interoperability, vendors should register their extension namespaces:

| Vendor ID | Vendor Name | Schema URL | Description |
|-----------|-------------|------------|-------------|
| `locus_robotics` | Locus Robotics | `https://locus.io/schemas/udm-ext/v1` | Warehouse AMR extensions |
| `boston_dynamics` | Boston Dynamics | `https://bostondynamics.com/schemas/udm-ext/v1` | Spot, Stretch extensions |
| `universal_robots` | Universal Robots | `https://universal-robots.com/schemas/udm-ext/v1` | UR cobot extensions |
| `tesla_optimus` | Tesla | `https://tesla.com/schemas/udm-ext/v1` | Optimus humanoid extensions |
| `nvidia_isaac` | NVIDIA | `https://nvidia.com/schemas/isaac-udm-ext/v1` | Isaac Sim integration |
| `ros2` | ROS 2 Community | `https://ros.org/schemas/udm-ext/v1` | ROS 2 message mappings |

*To register a vendor extension namespace, open a pull request against this specification.*

---

## Appendix C: Units Reference

| Quantity | SI Unit | UDM Suffix | Notes |
|----------|---------|------------|-------|
| Distance | meters | `_m` | All distances in meters |
| Velocity (linear) | m/s | `_mps` | Meters per second |
| Velocity (angular) | °/s | `_dps` | Degrees per second |
| Acceleration (linear) | m/s² | `_mps2` | Meters per second squared |
| Acceleration (angular) | °/s² | `_dps2` | Degrees per second squared |
| Angle | degrees | `_deg` | 0-360 for heading, ±180 for orientation |
| Angle (joints) | radians | `_rad` | For precision joint control |
| Force | Newtons | `_n` | |
| Torque | Newton-meters | `_nm` | |
| Mass | kilograms | `_kg` | |
| Temperature | Celsius | `_c` | |
| Time duration | seconds | `_sec` | |
| Percentage | 0-100 | `_pct` | Not 0-1 |
| Voltage | Volts | `_v` | |
| Current | Amperes | `_a` | |
| Power | Watts | `_w` | |
| Energy | Watt-hours | `_wh` | |
| Pressure | bar | `_bar` | Also `_kpa`, `_hpa` |
| Frequency | Hertz | `_hz` | |
| Data rate | Mbps | `_mbps` | |
| Sound level | decibels | `_db` | |
| Illuminance | lux | `_lux` | |

---

**Document Version:** 0.0.1  
**Last Updated:** January 2, 2026  
**Maintainer:** UDM Specification Authors