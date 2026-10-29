# Core Envelope


Every UDM event is wrapped in a standard envelope:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `udm_version` | string | Yes | Schema version (e.g., "0.0.1") |
| `event_id` | string | Yes | Globally unique event identifier (UUID v7 recommended) |
| `event_type` | string | Yes | Event classification (see Event Types) |
| `source_id` | string | Yes | Unique identifier of the data source (robot, sensor, system) |
| `source_type` | string | Yes | Source classification (see Source Types) |
| `captured_at` | string | Yes | ISO 8601 timestamp when data was captured at source |
| `received_at` | string | No | ISO 8601 timestamp when data was received by the ingest layer |
| `sent_at` | string | No | ISO 8601 timestamp when data was sent from source |
| `sequence_num` | integer | No | Monotonic sequence number from source |
| `session_id` | string | No | Session/run identifier for grouping events |

---

