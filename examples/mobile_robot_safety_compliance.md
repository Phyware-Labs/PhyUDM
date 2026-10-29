# UDM for Mobile Robot Safety Compliance

**Document Version:** 1.0.0  
**Date:** January 22, 2026  
**Status:** Reference Implementation Guide

---

## Overview

This document demonstrates how the Unified Data Model (UDM) and any conforming SDK can be used to track, monitor, and ensure compliance with key industry safety standards for mobile robotic platforms, including:

- **ANSI/ITSDF B56.5** – Safety standard for driverless automatic guided industrial vehicles
- **ISO 3691-4** – Safety of industrial trucks - driverless industrial trucks
- **ANSI/RIA R15.08 Part 1** – Safety requirements for industrial mobile robots
- **ANSI/RIA R15.08 Part 2** – Safety requirements for industrial mobile robot systems and applications

---

## Why Use UDM for Safety Compliance?

### 1. **Comprehensive Audit Trail**

Safety standards require documentation and traceability. UDM-based tooling provides:

- Immutable event records with cryptographic provenance
- Timestamped telemetry at microsecond precision
- Complete chain-of-custody from capture to storage
- Session-based grouping for incident reconstruction

### 2. **Real-Time Safety Monitoring**

Unlike periodic manual inspections, UDM-based continuous monitoring enables:

- Continuous telemetry streaming at configurable intervals (1-100+ Hz)
- Immediate detection of safety threshold violations
- Automated alerts when parameters drift outside safe ranges
- Historical trend analysis for predictive maintenance

### 3. **Cross-Platform Normalization**

Different AGV/AMR manufacturers produce different data formats. UDM:

- Normalizes all telemetry to a canonical schema
- Enables fleet-wide safety analytics across mixed vendors
- Supports vendor extensions without schema conflicts
- Allows apples-to-apples comparison of safety metrics

### 4. **Regulatory Reporting**

When incidents occur, investigators need data. UDM provides:

- Queryable event database for incident reconstruction
- Export capabilities for regulatory submissions
- Tamper-evident records for legal defensibility
- Integration with compliance management systems

---

## Standards Mapping to UDM Domains

### B56.5 Part II (User Requirements) → UDM Mapping

| B56.5 Requirement | UDM Domain | Key Fields | Event Types |
|-------------------|------------|------------|-------------|
| Battery safety monitoring | `power` | `battery.state_of_charge_pct`, `battery.temperature_c`, `battery.voltage_v`, `battery.chemistry` | `power.low_battery`, `power.charging_started` |
| Charging station compliance | `power`, `environment` | `charging.is_charging`, `charging.charger_id`, `charging_stations[]` | `power.charging_completed`, `environment.charging_station_docked` |
| Guidepath adherence | `navigation`, `location` | `path.deviation_m`, `localization.status`, `local.x_m/y_m` | `navigation.path_blocked`, `navigation.rerouting` |
| Clearance maintenance | `perception`, `safety` | `obstacles.nearest_distance_m`, `proximity.zones[]` | `safety.warning`, `safety.violation` |
| Floor surface conditions | `perception`, `navigation` | `surface.type`, `surface.condition`, `surface.friction_coefficient` | `sensor.degraded` |
| Safety feature status | `safety`, `operational` | `e_stop_active`, `safety_state`, `enabled` | `safety.e_stop`, `state.transition` |

### B56.5 Part III (Manufacturer Requirements) → UDM Mapping

| B56.5 Requirement | UDM Domain | Key Fields | Event Types |
|-------------------|------------|------------|-------------|
| Load capacity monitoring | `payload` | `total_weight_kg`, `capacity_pct`, `load_status` | `payload.loaded`, `payload.shifted` |
| Braking system status | `actuators`, `safety` | `brakes[]`, `brake_type`, `brake_state` | `safety.protective_stop`, `safety.e_stop` |
| Speed control on grades | `motion`, `location` | `linear_velocity.speed_mps`, `pitch_deg`, `commanded_velocity` | `safety.warning` |
| Stability monitoring | `motion`, `perception` | `roll_deg`, `pitch_deg`, `linear_acceleration` | `safety.violation` |
| Warning device activation | `operational`, `safety` | `warnings[]`, `indicators.visual`, `indicators.audible` | `state.transition` |
| Sensor functionality | `perception`, `maintenance` | `lidar[]`, `cameras[]`, `health_score`, `diagnostics[]` | `sensor.degraded`, `sensor.failed`, `sensor.recovered` |

### R15.08 (Industrial Mobile Robots) → UDM Mapping

| R15.08 Requirement | UDM Domain | Key Fields | Event Types |
|-------------------|------------|------------|-------------|
| Hazard zone detection | `safety`, `perception` | `proximity.zones[]`, `detections[]` | `safety.collaborative_mode_entered`, `interaction.user_present` |
| Human detection | `perception`, `hri` | `tracked_humans[]`, `interaction_state` | `interaction.gesture_detected`, `safety.contact_detected` |
| Emergency stop function | `safety`, `operational` | `e_stop.triggered`, `e_stop.source`, `safety_state` | `safety.e_stop` |
| Protective stop | `safety` | `safety_state`, `protective_stop_reason` | `safety.protective_stop` |
| Speed limitation | `motion`, `safety` | `speed_limit_mps`, `commanded_velocity`, `actual_velocity` | `safety.warning` |
| Risk assessment data | `safety`, `compliance` | `risk_assessment`, `functional_safety`, `sil_level` | `safety.violation` |

---

## Concrete UDM Examples

### Example 1: Battery Safety Monitoring (B56.5 Compliance)

This example shows how to capture battery telemetry for Li-ion safety compliance, including thermal monitoring to prevent thermal runaway events.

```json
{
  "udm_version": "0.0.3",
  "event_id": "evt-7f3a8c21-4b5e-4d9f-a1c3-2e8b9d0f1a2b",
  "event_type": "telemetry.periodic",
  "source_id": "agv-warehouse-042",
  "source_type": "agv",
  "captured_at": "2026-01-22T14:35:22.123456Z",
  "session_id": "session-20260122-shift2",
  
  "identity": {
    "source_id": "agv-warehouse-042",
    "source_type": "agv",
    "source_name": "Forklift AGV #42",
    "platform": "ACME Industrial AGV",
    "platform_version": "3.2.1",
    "serial_number": "AGV-2024-00042",
    "fleet_id": "warehouse-fleet-a",
    "site_id": "site-chicago-dc1",
    "organization_id": "org-acme-logistics"
  },
  
  "power": {
    "battery": {
      "state_of_charge_pct": 72.5,
      "state_of_health_pct": 94.2,
      "voltage_v": 352.4,
      "current_a": -45.2,
      "temperature_c": 38.5,
      "time_to_empty_min": 185,
      "chemistry": "lithium_ion",
      "cell_count": 96,
      "cell_balance_status": "balanced",
      "min_cell_voltage_v": 3.65,
      "max_cell_voltage_v": 3.68
    },
    "charging": {
      "is_charging": false,
      "last_charge_completed_at": "2026-01-22T06:15:00Z",
      "charger_id": null
    },
    "power_state": "discharging",
    "power_consumption_w": 15925.0
  },
  
  "thermal": {
    "thermal_state": "normal",
    "components": [
      {
        "component_id": "battery_pack_main",
        "component_type": "battery",
        "temperature_c": 38.5,
        "min_temp_c": 10.0,
        "max_temp_c": 45.0,
        "warning_temp_c": 42.0,
        "critical_temp_c": 50.0
      },
      {
        "component_id": "battery_pack_aux",
        "component_type": "battery",
        "temperature_c": 36.2,
        "min_temp_c": 10.0,
        "max_temp_c": 45.0
      }
    ],
    "cooling_system": {
      "active": true,
      "mode": "active_cooling",
      "fan_speed_rpm": 2400
    }
  },
  
  "compliance": {
    "certifications": [
      {
        "standard": "ANSI/ITSDF B56.5",
        "version": "2019",
        "certified": true,
        "expiry_date": "2027-06-15"
      },
      {
        "standard": "NFPA 505",
        "version": "2021",
        "certified": true,
        "expiry_date": "2027-06-15"
      }
    ]
  }
}
```

**Why This Matters for B56.5:**
- Battery temperature monitoring is critical for Li-ion safety (prevents thermal runaway)
- Cell-level voltage monitoring detects imbalances that can cause fires
- Charging state tracking ensures compliance with NFPA 505 requirements
- Historical data enables predictive maintenance before failures occur

---

### Example 2: Braking System Compliance (B56.5 Part III)

B56.5 requires three separate braking functions: emergency, parking, and service brakes. This example shows how to track all three.

```json
{
  "udm_version": "0.0.3",
  "event_id": "evt-8a4b9c32-5c6f-4e0a-b2d4-3f9c0e1a3b4c",
  "event_type": "telemetry.periodic",
  "source_id": "agv-warehouse-042",
  "source_type": "agv",
  "captured_at": "2026-01-22T14:35:22.234567Z",
  "session_id": "session-20260122-shift2",
  
  "actuators": {
    "brakes": [
      {
        "brake_id": "brake_service_front",
        "brake_type": "service",
        "location": "front_axle",
        "state": "released",
        "wear_pct": 15.2,
        "pad_thickness_mm": 8.5,
        "min_pad_thickness_mm": 2.0,
        "last_inspection_date": "2026-01-15",
        "temperature_c": 45.0
      },
      {
        "brake_id": "brake_service_rear",
        "brake_type": "service",
        "location": "rear_axle",
        "state": "released",
        "wear_pct": 12.8,
        "pad_thickness_mm": 8.7,
        "min_pad_thickness_mm": 2.0,
        "last_inspection_date": "2026-01-15",
        "temperature_c": 42.0
      },
      {
        "brake_id": "brake_parking",
        "brake_type": "parking",
        "location": "rear_axle",
        "state": "released",
        "actuator_type": "spring_applied_hydraulic_release",
        "spring_tension_healthy": true
      },
      {
        "brake_id": "brake_emergency",
        "brake_type": "emergency",
        "location": "all_wheels",
        "state": "ready",
        "actuator_type": "mechanical",
        "last_test_date": "2026-01-20",
        "test_result": "pass",
        "stopping_distance_m": 1.2,
        "rated_stopping_distance_m": 1.5
      }
    ]
  },
  
  "motion": {
    "linear_velocity": {
      "speed_mps": 1.5,
      "vx_mps": 1.5,
      "vy_mps": 0.0,
      "vz_mps": 0.0
    },
    "commanded_velocity": {
      "speed_mps": 1.5,
      "max_speed_mps": 2.0
    },
    "motion_state": "moving_forward"
  },
  
  "safety": {
    "safety_state": "normal",
    "e_stop": {
      "triggered": false,
      "sources": [
        {"source_id": "e_stop_panel", "state": "released"},
        {"source_id": "e_stop_remote", "state": "released"},
        {"source_id": "e_stop_software", "state": "released"}
      ]
    },
    "stopping_distance": {
      "current_m": 0.8,
      "rated_m": 1.5,
      "surface_friction_coefficient": 0.65,
      "load_factor": 0.85
    }
  },
  
  "environment_interaction": {
    "surface": {
      "type": "concrete",
      "condition": "dry",
      "friction_coefficient": 0.65,
      "grade_pct": 0.5
    }
  }
}
```

**Why This Matters for B56.5:**
- All three brake types (service, parking, emergency) are individually tracked
- Brake wear monitoring enables predictive maintenance
- Stopping distance calculation accounts for load and surface conditions
- Emergency brake testing records provide audit trail for inspections

---

### Example 3: Hazard Zone Detection (R15.08 Compliance)

R15.08 requires mobile robots to detect and react to humans in their operating area. This example shows zone-based safety monitoring.

```json
{
  "udm_version": "0.0.3",
  "event_id": "evt-9b5c0d43-6d7a-4f1b-c3e5-4a0d1f2b4c5d",
  "event_type": "safety.warning",
  "source_id": "amr-picking-007",
  "source_type": "amr",
  "captured_at": "2026-01-22T14:35:22.345678Z",
  "session_id": "session-20260122-shift2",
  
  "identity": {
    "source_id": "amr-picking-007",
    "source_type": "amr",
    "source_name": "Picking AMR #7",
    "platform": "PickBot Pro",
    "fleet_id": "picking-fleet",
    "site_id": "site-chicago-dc1",
    "zone_id": "zone-aisle-14"
  },
  
  "location": {
    "coordinate_system": "local",
    "local": {
      "coordinate_frame": "map",
      "x_m": 45.23,
      "y_m": 12.87,
      "z_m": 0.0,
      "yaw_deg": 90.5
    },
    "semantic": {
      "zone_name": "Aisle 14",
      "zone_type": "picking_aisle",
      "area_name": "Zone B - Electronics"
    }
  },
  
  "safety": {
    "safety_state": "warning",
    "proximity": {
      "zones": [
        {
          "zone_id": "zone_emergency",
          "zone_type": "emergency_stop",
          "shape": "cylinder",
          "radius_m": 0.3,
          "occupied": false,
          "triggering_object": null
        },
        {
          "zone_id": "zone_protective",
          "zone_type": "protective_stop",
          "shape": "cylinder",
          "radius_m": 0.8,
          "occupied": false,
          "triggering_object": null
        },
        {
          "zone_id": "zone_warning",
          "zone_type": "reduced_speed",
          "shape": "sector",
          "radius_m": 2.5,
          "angle_deg": 180,
          "occupied": true,
          "triggering_object": {
            "object_type": "human",
            "tracking_id": "human-track-0042",
            "distance_m": 2.1,
            "bearing_deg": 15.0,
            "velocity_mps": 0.8,
            "approach_angle_deg": 165.0
          }
        },
        {
          "zone_id": "zone_awareness",
          "zone_type": "monitoring",
          "shape": "sector",
          "radius_m": 5.0,
          "angle_deg": 270,
          "occupied": true,
          "object_count": 2
        }
      ],
      "nearest_human_m": 2.1,
      "nearest_obstacle_m": 2.1
    },
    "speed_limiting": {
      "active": true,
      "reason": "human_in_warning_zone",
      "original_speed_mps": 2.0,
      "limited_speed_mps": 0.5
    }
  },
  
  "perception": {
    "lidar": [
      {
        "sensor_id": "lidar_front",
        "sensor_type": "2d_scanning",
        "status": "operational",
        "scan_rate_hz": 25,
        "range_m": 30.0,
        "angular_resolution_deg": 0.25,
        "detections_count": 847
      },
      {
        "sensor_id": "lidar_rear",
        "sensor_type": "2d_scanning",
        "status": "operational",
        "scan_rate_hz": 25,
        "range_m": 30.0,
        "detections_count": 523
      }
    ],
    "detections": [
      {
        "detection_id": "det-2026012214352234-001",
        "tracking_id": "human-track-0042",
        "object_type": "human",
        "object_class": "pedestrian",
        "detection_confidence": 0.94,
        "pose": {
          "position": {"x_m": 47.15, "y_m": 13.42, "z_m": 0.0},
          "frame_id": "map"
        },
        "velocity": {"vx_mps": -0.6, "vy_mps": 0.5},
        "bounding_box": {
          "center": {"x_m": 47.15, "y_m": 13.42, "z_m": 0.85},
          "dimensions": {"length_m": 0.5, "width_m": 0.4, "height_m": 1.7}
        }
      },
      {
        "detection_id": "det-2026012214352234-002",
        "tracking_id": "human-track-0043",
        "object_type": "human",
        "object_class": "pedestrian",
        "detection_confidence": 0.87,
        "pose": {
          "position": {"x_m": 49.80, "y_m": 15.20, "z_m": 0.0},
          "frame_id": "map"
        }
      }
    ]
  },
  
  "hri": {
    "interaction_state": "aware",
    "tracked_humans": [
      {
        "tracking_id": "human-track-0042",
        "distance_m": 2.1,
        "relative_velocity_mps": -0.3,
        "predicted_collision_time_sec": null,
        "attention_state": "unknown",
        "body_orientation_deg": 180
      },
      {
        "tracking_id": "human-track-0043",
        "distance_m": 4.8,
        "relative_velocity_mps": 0.0,
        "attention_state": "unknown"
      }
    ]
  },
  
  "motion": {
    "linear_velocity": {
      "speed_mps": 0.5
    },
    "commanded_velocity": {
      "speed_mps": 0.5,
      "max_speed_mps": 2.0
    },
    "motion_state": "moving_forward"
  },
  
  "compliance": {
    "certifications": [
      {
        "standard": "ANSI/RIA R15.08-1",
        "version": "2020",
        "certified": true
      },
      {
        "standard": "ANSI/RIA R15.08-2",
        "version": "2020",
        "certified": true
      }
    ],
    "functional_safety": {
      "sil_level": "SIL2",
      "pl_level": "PLd",
      "safety_function": "speed_reduction_on_human_detection"
    }
  }
}
```

**Why This Matters for R15.08:**
- Multi-zone safety architecture matches standard requirements
- Human detection with tracking enables predictive collision avoidance
- Speed limiting provides documented safety response
- Functional safety level (SIL/PL) tracking demonstrates compliance

---

### Example 4: Emergency Stop Event (B56.5 & R15.08)

When an emergency stop occurs, complete documentation is critical for incident investigation and compliance verification.

```json
{
  "udm_version": "0.0.3",
  "event_id": "evt-0c6d1e54-7e8b-4a2c-d4f6-5b1e2a3c5d6e",
  "event_type": "safety.e_stop",
  "source_id": "agv-warehouse-042",
  "source_type": "agv",
  "captured_at": "2026-01-22T14:35:22.456789Z",
  "session_id": "session-20260122-shift2",
  
  "identity": {
    "source_id": "agv-warehouse-042",
    "source_type": "agv",
    "source_name": "Forklift AGV #42",
    "fleet_id": "warehouse-fleet-a",
    "site_id": "site-chicago-dc1",
    "zone_id": "zone-loading-dock-3"
  },
  
  "location": {
    "coordinate_system": "local",
    "local": {
      "coordinate_frame": "map",
      "x_m": 120.45,
      "y_m": 35.67,
      "z_m": 0.0,
      "yaw_deg": 270.0
    },
    "semantic": {
      "zone_name": "Loading Dock 3",
      "zone_type": "loading_dock"
    }
  },
  
  "safety": {
    "safety_state": "emergency_stop",
    "e_stop": {
      "triggered": true,
      "triggered_at": "2026-01-22T14:35:22.456789Z",
      "source": "proximity_sensor",
      "sources": [
        {"source_id": "e_stop_panel", "state": "released"},
        {"source_id": "e_stop_remote", "state": "released"},
        {"source_id": "proximity_sensor", "state": "triggered"},
        {"source_id": "e_stop_software", "state": "triggered"}
      ],
      "reason": "object_in_emergency_zone",
      "triggering_detection": {
        "tracking_id": "human-track-0089",
        "object_type": "human",
        "distance_m": 0.25,
        "zone_id": "zone_emergency"
      }
    },
    "proximity": {
      "zones": [
        {
          "zone_id": "zone_emergency",
          "zone_type": "emergency_stop",
          "radius_m": 0.3,
          "occupied": true,
          "triggering_object": {
            "tracking_id": "human-track-0089",
            "object_type": "human",
            "distance_m": 0.25
          }
        }
      ],
      "nearest_human_m": 0.25
    },
    "stop_performance": {
      "pre_stop_speed_mps": 1.2,
      "stopping_distance_m": 0.35,
      "stopping_time_ms": 420,
      "deceleration_mps2": 2.86
    }
  },
  
  "motion": {
    "linear_velocity": {
      "speed_mps": 0.0
    },
    "motion_state": "stationary",
    "pre_event_velocity": {
      "speed_mps": 1.2
    }
  },
  
  "actuators": {
    "brakes": [
      {
        "brake_id": "brake_emergency",
        "brake_type": "emergency",
        "state": "engaged",
        "engaged_at": "2026-01-22T14:35:22.456789Z"
      },
      {
        "brake_id": "brake_service_front",
        "brake_type": "service",
        "state": "engaged"
      },
      {
        "brake_id": "brake_service_rear",
        "brake_type": "service",
        "state": "engaged"
      }
    ]
  },
  
  "operational": {
    "mode": "emergency",
    "state": "emergency_stop",
    "previous_mode": "autonomous",
    "previous_state": "executing_task",
    "task": {
      "task_id": "task-pallet-move-2847",
      "task_type": "pallet_transport",
      "status": "interrupted",
      "interrupted_at": "2026-01-22T14:35:22.456789Z",
      "interrupt_reason": "emergency_stop"
    }
  },
  
  "perception": {
    "detections": [
      {
        "detection_id": "det-critical-001",
        "tracking_id": "human-track-0089",
        "object_type": "human",
        "object_class": "pedestrian",
        "detection_confidence": 0.98,
        "pose": {
          "position": {"x_m": 120.20, "y_m": 35.67, "z_m": 0.0},
          "frame_id": "map"
        },
        "critical_detection": true
      }
    ],
    "lidar": [
      {
        "sensor_id": "lidar_front",
        "status": "operational",
        "scan_at_event": {
          "timestamp": "2026-01-22T14:35:22.450000Z",
          "nearest_point_m": 0.25,
          "nearest_point_angle_deg": 0.0
        }
      }
    ]
  },
  
  "context": {
    "time": {
      "local_time": "2026-01-22T08:35:22-06:00",
      "shift": "day_shift",
      "is_break_time": false
    },
    "facility": {
      "area_occupancy": "high",
      "nearby_personnel_count": 4
    }
  }
}
```

**Why This Matters:**
- Complete incident documentation for regulatory reporting
- Stop performance metrics verify braking compliance
- Detection chain proves safety systems functioned correctly
- Task context enables root cause analysis

---

### Example 5: Guidepath Deviation Alert (B56.5 Compliance)

AGVs must follow predefined guidepaths. This example shows how to track deviations.

```json
{
  "udm_version": "0.0.3",
  "event_id": "evt-1d7e2f65-8f9c-4b3d-e5a7-6c2f3b4d6e7f",
  "event_type": "navigation.rerouting",
  "source_id": "agv-warehouse-042",
  "source_type": "agv",
  "captured_at": "2026-01-22T14:35:22.567890Z",
  "session_id": "session-20260122-shift2",
  
  "identity": {
    "source_id": "agv-warehouse-042",
    "source_type": "agv",
    "fleet_id": "warehouse-fleet-a",
    "site_id": "site-chicago-dc1"
  },
  
  "location": {
    "local": {
      "coordinate_frame": "map",
      "x_m": 78.45,
      "y_m": 22.30,
      "z_m": 0.0,
      "yaw_deg": 180.0
    }
  },
  
  "navigation": {
    "localization": {
      "status": "localized",
      "method": "fusion",
      "confidence": 0.97,
      "position_uncertainty_m": 0.05
    },
    "path": {
      "path_id": "guidepath-main-corridor-south",
      "path_type": "guidepath",
      "status": "deviation",
      "deviation_m": 0.15,
      "max_allowed_deviation_m": 0.10,
      "deviation_reason": "obstacle_avoidance",
      "on_guidepath": false,
      "returning_to_guidepath": true,
      "estimated_return_distance_m": 2.5
    },
    "obstacles": {
      "count": 1,
      "nearest_distance_m": 1.2,
      "blocking_path": true,
      "obstacle_details": [
        {
          "obstacle_id": "obs-temp-001",
          "obstacle_type": "pallet",
          "position": {"x_m": 79.00, "y_m": 22.00},
          "blocking_guidepath": true,
          "expected_clearance_time_sec": null
        }
      ]
    },
    "guidepath": {
      "guidepath_id": "guidepath-main-corridor-south",
      "segment_id": "segment-42",
      "clearance_left_m": 0.6,
      "clearance_right_m": 0.45,
      "required_clearance_m": 0.5,
      "clearance_adequate": false
    }
  },
  
  "safety": {
    "safety_state": "warning",
    "warnings": [
      {
        "warning_id": "warn-guidepath-deviation",
        "warning_type": "guidepath_deviation",
        "severity": "medium",
        "message": "AGV deviated from guidepath due to obstacle",
        "acknowledged": false
      },
      {
        "warning_id": "warn-clearance-inadequate",
        "warning_type": "clearance_violation",
        "severity": "low",
        "message": "Right-side clearance below 0.5m minimum"
      }
    ]
  },
  
  "motion": {
    "linear_velocity": {
      "speed_mps": 0.8
    },
    "commanded_velocity": {
      "speed_mps": 0.8,
      "max_speed_mps": 1.5
    },
    "motion_state": "moving_forward"
  }
}
```

**Why This Matters for B56.5:**
- Guidepath adherence is a core AGV requirement
- Deviation tracking with reasons provides audit trail
- Clearance monitoring ensures worker safety
- Obstacle documentation supports facility operations

---

## SDK Integration Example

### Python Code for Safety Telemetry Capture

```python
"""
Example: Capturing AGV safety telemetry for B56.5 compliance
using a hypothetical UDM-conforming SDK (`udm_sdk`).
"""

from udm_sdk import (
    UdmAgent,
    UDMEventBuilder,
    SourceType,
    PeriodicEmitter,
    OnChangeEmitter,
    EventEmitter
)
from udm_sdk.domains import (
    PowerDomain,
    SafetyDomain,
    ActuatorsDomain,
    NavigationDomain,
    ComplianceDomain
)

# Initialize the UDM agent
agent = UdmAgent.from_config("udm-agent.yaml")

# Configure periodic telemetry for battery monitoring (B56.5 requirement)
battery_emitter = PeriodicEmitter(
    interval_ms=1000,  # 1 Hz for battery status
    domains=["power", "thermal"]
)

# Configure high-frequency safety zone monitoring (R15.08 requirement)
safety_emitter = PeriodicEmitter(
    interval_ms=100,  # 10 Hz for safety-critical data
    domains=["safety", "perception", "motion"]
)

# Configure change-based emitter for brake state changes
brake_emitter = OnChangeEmitter(
    fields=[
        "actuators.brakes[*].state",
        "actuators.brakes[*].wear_pct"
    ],
    threshold=0.01  # 1% change triggers event
)

# Configure event-triggered emitter for safety events
safety_event_emitter = EventEmitter()

# Register safety event handlers
@safety_event_emitter.on("emergency_stop_triggered")
async def handle_e_stop(context):
    """Capture comprehensive data on E-stop events."""
    event = (
        UDMEventBuilder(
            source_id=context.source_id,
            source_type=SourceType.AGV
        )
        .with_event_type("safety.e_stop")
        .with_safety(
            safety_state="emergency_stop",
            e_stop={
                "triggered": True,
                "source": context.trigger_source,
                "reason": context.reason
            },
            stop_performance={
                "pre_stop_speed_mps": context.pre_stop_speed,
                "stopping_distance_m": context.stopping_distance,
                "stopping_time_ms": context.stopping_time
            }
        )
        .with_motion(
            speed_mps=0.0,
            motion_state="stationary",
            pre_event_velocity={"speed_mps": context.pre_stop_speed}
        )
        .with_location(context.current_location)
        .with_perception(context.sensor_snapshot)
        .build()
    )
    
    # High-priority immediate transmission
    await agent.emit(event, priority="critical")


@safety_event_emitter.on("proximity_zone_entered")
async def handle_zone_entry(context):
    """Track when objects enter safety zones."""
    zone_type = context.zone.zone_type
    
    if zone_type == "emergency_stop":
        event_type = "safety.violation"
        priority = "critical"
    elif zone_type == "protective_stop":
        event_type = "safety.warning"
        priority = "high"
    else:
        event_type = "telemetry.on_change"
        priority = "normal"
    
    event = (
        UDMEventBuilder(
            source_id=context.source_id,
            source_type=SourceType.AGV
        )
        .with_event_type(event_type)
        .with_safety(
            proximity={
                "zones": context.all_zones,
                "nearest_human_m": context.nearest_human,
                "nearest_obstacle_m": context.nearest_obstacle
            }
        )
        .build()
    )
    
    await agent.emit(event, priority=priority)


# Register emitters with the agent
agent.add_emitter(battery_emitter)
agent.add_emitter(safety_emitter)
agent.add_emitter(brake_emitter)
agent.add_emitter(safety_event_emitter)

# Start the agent
async def main():
    await agent.start()
    
    # Agent runs continuously, capturing and transmitting telemetry
    # Press Ctrl+C to stop
    try:
        await agent.wait_forever()
    except KeyboardInterrupt:
        await agent.stop()

if __name__ == "__main__":
    import asyncio
    asyncio.run(main())
```

---

## Compliance Reporting with UDM

### Automated Compliance Reports

UDM-based platforms enable automated generation of compliance reports by querying the telemetry database:

| Report Type | Standards | Data Sources | Frequency |
|-------------|-----------|--------------|-----------|
| Battery Safety Report | B56.5, NFPA 505 | `power`, `thermal` domains | Daily |
| Braking System Report | B56.5 | `actuators.brakes`, `safety.stop_performance` | Weekly |
| Human Interaction Report | R15.08 | `safety.proximity`, `hri`, `perception.detections` | Daily |
| E-Stop Incident Report | B56.5, R15.08 | `safety.e_stop` events | Per incident |
| Guidepath Compliance Report | B56.5 | `navigation.path`, `navigation.guidepath` | Weekly |
| Sensor Health Report | B56.5, R15.08 | `perception`, `maintenance.diagnostics` | Daily |

### Example Query: E-Stop Events Last 30 Days

```sql
-- UDM Analytics Query
SELECT 
    captured_at,
    source_id,
    safety.e_stop.source AS trigger_source,
    safety.e_stop.reason AS stop_reason,
    safety.stop_performance.stopping_distance_m,
    safety.stop_performance.pre_stop_speed_mps,
    location.semantic.zone_name AS location_zone
FROM 
    udm_events
WHERE 
    event_type = 'safety.e_stop'
    AND captured_at >= NOW() - INTERVAL '30 days'
    AND identity.site_id = 'site-chicago-dc1'
ORDER BY 
    captured_at DESC;
```

---

## Summary: Standards Compliance Matrix

| Standard | Section | UDM Coverage | Key Domains |
|----------|---------|--------------|-------------|
| **B56.5 Part II** | Battery Safety | ✅ Full | `power`, `thermal`, `compliance` |
| | Guidepath Marking | ✅ Full | `navigation`, `location` |
| | Clearance Requirements | ✅ Full | `navigation`, `safety`, `perception` |
| | Safety Feature Status | ✅ Full | `safety`, `operational` |
| | Floor Surface Conditions | ✅ Full | `environment_interaction`, `perception` |
| **B56.5 Part III** | Braking Systems | ✅ Full | `actuators`, `safety`, `motion` |
| | Load Capacity | ✅ Full | `payload` |
| | Speed Control | ✅ Full | `motion`, `safety` |
| | Warning Devices | ✅ Full | `operational`, `safety` |
| | Sensor Functionality | ✅ Full | `perception`, `maintenance` |
| **R15.08 Part 1** | Hazard Zone Detection | ✅ Full | `safety`, `perception` |
| | Human Detection | ✅ Full | `perception`, `hri` |
| | Emergency Stop | ✅ Full | `safety`, `actuators` |
| | Speed Limitation | ✅ Full | `motion`, `safety` |
| **R15.08 Part 2** | Risk Assessment | ✅ Full | `compliance`, `safety` |
| | System Integration | ✅ Full | All domains |
| **ISO 3691-4** | Operator Safety | ✅ Full | `safety`, `hri` |
| | System Design | ✅ Full | `compliance`, `identity` |

---

## Conclusion

The Unified Data Model provides comprehensive coverage for mobile robot safety compliance. By capturing telemetry across all relevant domains, organizations can:

1. **Demonstrate Compliance** – Provide auditors with complete, timestamped evidence
2. **Prevent Incidents** – Use real-time monitoring to catch issues before they become incidents
3. **Investigate Thoroughly** – Reconstruct any incident with full sensor and state data
4. **Improve Continuously** – Analyze trends to enhance safety over time

For questions or implementation support, open an issue in this repository.

---

**Document Maintainer:** UDM Specification Authors  
**Last Updated:** January 22, 2026
