# Object References


Physical objects (packages, tools, pallets, parts, etc.) that robots perceive, manipulate, or transport are referenced consistently across domains using the `object_ref` pattern.

### ID Semantics

| ID Type | Scope | Description |
|---------|-------|-------------|
| `detection_id` | Single frame | Unique ID for one perception output (ephemeral, per-frame) |
| `tracking_id` | Perception track | Short-lived ID for a tracked object; may change after occlusion/re-identification |
| `object_id` | Session | Stable ID for a physical object within a `session_id`; links perception, manipulation, and payload |
| `item_id` | External system | Business identifier from WMS, ERP, or asset management (e.g., order number, SKU, asset tag) |

### object_ref Schema

When referencing a physical object, use the following structure:

```json
{
  "object_id": "obj-123",
  "item_id": "ORD-2026-00456",
  "object_type": "package",
  "object_class": "cardboard_box",
  "dimensions_m": { "length": 0.4, "width": 0.3, "height": 0.2 },
  "mass_kg": 2.5,
  "tracking_id": "track-001",
  "detection_confidence": 0.95,
  "barcode": "1234567890123",
  "rfid_tag": "RFID-ABC-123",
  "asset_id": "ASSET-00789",
  "hazards": ["fragile"],
  "pose": {
    "x_m": 1.2,
    "y_m": 0.5,
    "z_m": 0.8,
    "roll_deg": 0.0,
    "pitch_deg": 0.0,
    "yaw_deg": 45.0
  },
  "frame_id": "world"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `object_id` | string | Session-stable ID linking perception → manipulation → payload |
| `item_id` | string | External business identifier (WMS order, SKU, asset tag) |
| `object_type` | string | High-level type: `package`, `pallet`, `tote`, `bin`, `tool`, `part`, `container`, `unknown` |
| `object_class` | string | Specific classification from perception (e.g., `cardboard_box`, `plastic_tote`) |
| `dimensions_m` | object | Physical dimensions: `length`, `width`, `height` |
| `mass_kg` | float | Known or estimated mass |
| `tracking_id` | string | Current perception tracking ID (may change) |
| `detection_confidence` | float | Confidence score from perception (0.0–1.0) |
| `barcode` | string | Barcode value if scanned |
| `rfid_tag` | string | RFID tag ID if detected |
| `asset_id` | string | Asset management system ID |
| `hazards` | array[string] | Hazard labels: `fragile`, `flammable`, `corrosive`, `heavy`, `temperature_sensitive` |
| `pose` | object | Object pose if known (position + orientation) |
| `frame_id` | string | Reference frame for pose |

**Usage:** Not all fields are required. Include `object_id` when linking across domains. Include `item_id` when correlating with business systems.

---

