# OpenTelemetry Compatibility Layer


Conforming implementations MAY provide an OpenTelemetry exporter that maps UDM events to OTel primitives as follows:

| UDM Domain | OTel Primitive | Mapping |
|------------|----------------|---------|
| `event_type: telemetry.*` | OTel Metrics | Gauge/Counter for numeric fields |
| `event_type: task.*` | OTel Spans | Span per task lifecycle |
| `event_type: state.*` | OTel Logs | Structured log event |
| `event_type: safety.*` | OTel Logs + Metrics | Log event + violation counter |
| `identity.*` | OTel Resource Attributes | Standard resource identification |
| `location.*` | OTel Attributes | `geo.lat`, `geo.lon`, custom attrs |

The UDM remains the canonical format for any conforming backend; the OTel exporter is an interoperability layer.

---

