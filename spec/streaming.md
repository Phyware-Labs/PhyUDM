# Streaming & Batching


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

