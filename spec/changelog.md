# Changelog (specification text)

This page mirrors the `## Changelog` section of the monolithic
[`udm-spec.md`](./udm-spec.md). For the canonical project changelog
(covering tooling, schema artifacts, and SDLC events alongside the
spec text), see the root [`CHANGELOG.md`](../CHANGELOG.md).


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

