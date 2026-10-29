# Design Principles


| Principle | Description |
|-----------|-------------|
| **Canonical Fields First** | Common robotics concepts (position, velocity, battery) have standard field names |
| **Hierarchical Domains** | Related fields are grouped into logical domains (identity, motion, power, etc.) |
| **Optional Everything** | No field is required except `event_id`, `source_id`, and `captured_at` |
| **Extensible by Design** | `extensions` namespace allows vendor/platform-specific data |
| **Immutable Events** | Each UDM record is immutable once created; updates create new events |
| **SI Units** | All physical quantities use SI units with explicit unit fields where ambiguous |
| **Temporal Precision** | Timestamps use ISO 8601 with microsecond precision and timezone |

---

