# 2. Location Domain

Physical location and coordinate systems.

```json
{
  "location": {
    "coordinate_system": "wgs84",
    "latitude": 41.8781,
    "longitude": -87.6298,
    "altitude_m": 182.5,
    "altitude_reference": "msl",
    "horizontal_accuracy_m": 0.5,
    "vertical_accuracy_m": 1.0,
    "heading_deg": 45.0,
    "heading_reference": "true_north",
    "floor": 2,
    "building": "warehouse-a",
    "local": {
      "coordinate_frame": "map",
      "x_m": 15.234,
      "y_m": 8.567,
      "z_m": 0.0,
      "roll_deg": 0.0,
      "pitch_deg": 0.0,
      "yaw_deg": 45.0
    },
    "grid": {
      "grid_id": "warehouse-grid-1",
      "cell_x": 15,
      "cell_y": 8,
      "cell_size_m": 1.0
    },
    "semantic": {
      "area": "picking-zone-a",
      "aisle": "A7",
      "position": "rack-15-slot-3"
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `coordinate_system` | string | Coordinate system: `wgs84`, `utm`, `local`, `grid` |
| `latitude` | float | WGS84 latitude (degrees) |
| `longitude` | float | WGS84 longitude (degrees) |
| `altitude_m` | float | Altitude (meters) |
| `altitude_reference` | string | Altitude reference: `msl`, `agl`, `ellipsoid` |
| `horizontal_accuracy_m` | float | Horizontal position accuracy (meters) |
| `vertical_accuracy_m` | float | Vertical position accuracy (meters) |
| `heading_deg` | float | Heading (degrees, 0-360) |
| `heading_reference` | string | Heading reference: `true_north`, `magnetic_north`, `grid_north` |
| `floor` | integer | Building floor number |
| `building` | string | Building identifier |
| `local` | object | Local coordinate frame position |
| `local.coordinate_frame` | string | Frame ID (e.g., `map`, `odom`, `base_link`) |
| `local.x_m`, `y_m`, `z_m` | float | Position in local frame (meters) |
| `local.roll_deg`, `pitch_deg`, `yaw_deg` | float | Orientation in local frame (degrees) |
| `grid` | object | Grid-based position |
| `semantic` | object | Semantic location labels |

---

