# 14. Context Domain

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

