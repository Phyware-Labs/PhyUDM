# Complete UDM Event Example


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

