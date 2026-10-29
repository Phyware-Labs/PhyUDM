# 15. Payload/Cargo Domain

Cargo, payload, and load management for logistics and delivery robots.

```json
{
  "payload": {
    "load_status": "loaded",
    "total_weight_kg": 25.5,
    "max_capacity_kg": 50.0,
    "center_of_gravity": {
      "x_m": 0.1,
      "y_m": 0.0,
      "z_m": 0.3
    },
    "compartments": [
      {
        "compartment_id": "main",
        "status": "occupied",
        "weight_kg": 25.5,
        "volume_used_pct": 60.0,
        "door_state": "closed",
        "locked": true,
        "temperature_c": 4.0,
        "temperature_setpoint_c": 4.0,
        "humidity_pct": 45.0
      }
    ],
    "items": [
      {
        "object_id": "obj-456",
        "item_id": "ORD-2026-00789",
        "object_type": "package",
        "object_class": "cardboard_box",
        "weight_kg": 5.2,
        "dimensions_m": { "length": 0.4, "width": 0.3, "height": 0.2 },
        "loaded_at": "2026-01-02T10:15:00Z",
        "destination": "dock-5",
        "fragile": false,
        "temperature_sensitive": true,
        "barcode": "1234567890123",
        "rfid_tag": "RFID-ABC-123",
        "asset_id": "ASSET-00123"
      }
    ],
    "load_shifted": false,
    "overweight": false,
    "secured": true
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `load_status` | string | Load state: `empty`, `loading`, `loaded`, `unloading` |
| `total_weight_kg` | float | Total payload weight (kg) |
| `max_capacity_kg` | float | Maximum payload capacity (kg) |
| `center_of_gravity` | object | Payload center of gravity offset |
| `compartments` | array | Individual storage compartments |
| `compartments[].temperature_c` | float | Compartment temperature for cold chain |
| `items` | array | Individual cargo items/packages (see `object_ref`) |
| `items[].object_id` | string | Session-stable ID linking to perception/manipulation |
| `items[].item_id` | string | External business identifier (WMS order, SKU, asset tag) |
| `items[].object_type` | string | Object type: `package`, `pallet`, `tote`, `bin`, `part`, etc. |
| `items[].object_class` | string | Specific classification from perception |
| `items[].dimensions_m` | object | Physical dimensions |
| `items[].asset_id` | string | Asset management system ID |
| `load_shifted` | boolean | Payload shift detected |
| `secured` | boolean | Payload properly secured |

---

