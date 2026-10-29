# 9. Actuators Domain

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

