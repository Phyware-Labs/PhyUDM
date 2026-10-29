# 8. Safety Domain

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

