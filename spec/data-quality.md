# Data Quality & Validation


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

