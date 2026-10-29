# 4. Power Domain

Battery, charging, and energy management.

```json
{
  "power": {
    "battery": {
      "state_of_charge_pct": 78.5,
      "state_of_health_pct": 95.0,
      "voltage_v": 48.2,
      "current_a": -12.5,
      "power_w": -602.5,
      "temperature_c": 32.0,
      "time_to_empty_min": 95,
      "time_to_full_min": null,
      "cycle_count": 342,
      "chemistry": "lifepo4",
      "capacity_ah": 100.0,
      "capacity_remaining_ah": 78.5
    },
    "charging": {
      "is_charging": false,
      "is_plugged_in": false,
      "charger_id": null,
      "charging_power_w": 0,
      "charging_mode": null
    },
    "power_state": "discharging",
    "power_consumption_w": 602.5,
    "power_budget_pct": 45.0,
    "energy_regenerated_wh": 12.3
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `battery.state_of_charge_pct` | float | Current charge level (0-100%) |
| `battery.state_of_health_pct` | float | Battery health (0-100%) |
| `battery.voltage_v` | float | Battery voltage (V) |
| `battery.current_a` | float | Current draw (A, negative = discharging) |
| `battery.temperature_c` | float | Battery temperature (°C) |
| `battery.time_to_empty_min` | integer | Estimated time to empty (minutes) |
| `battery.chemistry` | string | Battery chemistry type |
| `charging.is_charging` | boolean | Currently charging |
| `charging.charger_id` | string | Connected charger identifier |
| `power_state` | string | Power state classification |
| `power_consumption_w` | float | Current power consumption (W) |

**Power States:** `discharging`, `charging`, `charged`, `hibernating`, `emergency_power`, `external_power`

---

