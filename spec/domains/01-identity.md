# 1. Identity Domain

Identifies the source and its organizational context.

```json
{
  "identity": {
    "source_id": "robot-001",
    "source_type": "amr",
    "source_name": "Picker Alpha",
    "platform": "locus_robotics",
    "platform_version": "3.2.1",
    "firmware_version": "2024.12.01",
    "hardware_revision": "rev-c",
    "serial_number": "LR-2024-00001234",
    "fleet_id": "warehouse-east-fleet-1",
    "site_id": "site-chicago-dc1",
    "zone_id": "zone-aisle-7",
    "organization_id": "org-acme-corp",
    "tags": ["picker", "high-priority", "shift-1"]
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `source_id` | string | Unique source identifier (required) |
| `source_type` | string | Source type classification (required) |
| `source_name` | string | Human-readable name |
| `platform` | string | Platform/manufacturer identifier |
| `platform_version` | string | Platform software version |
| `firmware_version` | string | Firmware version |
| `hardware_revision` | string | Hardware revision |
| `serial_number` | string | Manufacturer serial number |
| `fleet_id` | string | Fleet grouping identifier |
| `site_id` | string | Physical site/facility identifier |
| `zone_id` | string | Current operational zone |
| `organization_id` | string | Customer/tenant organization |
| `tags` | array[string] | Arbitrary classification tags |

---

