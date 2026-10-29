# 20. Thermal Management Domain

Temperature monitoring and thermal control systems.

```json
{
  "thermal": {
    "thermal_state": "normal",
    "ambient_temperature_c": 25.0,
    "components": [
      {
        "component_id": "cpu_main",
        "temperature_c": 55.0,
        "max_temperature_c": 95.0,
        "throttling": false,
        "fan_speed_rpm": 2400
      },
      {
        "component_id": "motor_left",
        "temperature_c": 48.0,
        "max_temperature_c": 80.0,
        "throttling": false
      },
      {
        "component_id": "battery_pack",
        "temperature_c": 32.0,
        "max_temperature_c": 45.0,
        "min_temperature_c": 5.0,
        "heating_active": false,
        "cooling_active": false
      }
    ],
    "cooling_system": {
      "type": "active_air",
      "status": "running",
      "fan_count": 2,
      "total_airflow_cfm": 50.0,
      "coolant_temperature_c": null,
      "coolant_flow_lpm": null
    },
    "heating_system": {
      "enabled": true,
      "active": false,
      "power_w": 0
    },
    "enclosure": {
      "internal_temperature_c": 35.0,
      "ingress_protection": "IP65",
      "sealed": true
    },
    "thermal_limits": {
      "operating_min_c": -10.0,
      "operating_max_c": 50.0,
      "storage_min_c": -20.0,
      "storage_max_c": 60.0
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `thermal_state` | string | Overall state: `cold`, `normal`, `warm`, `hot`, `critical` |
| `components` | array | Per-component temperature status |
| `components[].throttling` | boolean | Performance throttling due to temperature |
| `cooling_system` | object | Active cooling status |
| `heating_system` | object | Cold-weather heating status |
| `enclosure` | object | Enclosure environmental protection |
| `thermal_limits` | object | Operating temperature specifications |

---

