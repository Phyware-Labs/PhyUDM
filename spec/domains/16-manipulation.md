# 16. Manipulation Domain

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

