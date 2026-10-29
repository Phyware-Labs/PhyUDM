# 3. Motion Domain

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

