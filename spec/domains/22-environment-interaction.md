# 22. Environment Interaction Domain

Physical environment interactions including doors, elevators, and infrastructure.

```json
{
  "environment_interaction": {
    "doors": [
      {
        "door_id": "door-A1",
        "type": "automatic_sliding",
        "state": "open",
        "controlled_by_robot": true,
        "request_pending": false,
        "access_granted": true,
        "authentication_method": "ble_beacon"
      }
    ],
    "elevators": {
      "current_elevator_id": null,
      "elevator_state": "not_in_elevator",
      "floor_requested": null,
      "current_floor": 1,
      "elevator_summoned": false,
      "doors_open": false,
      "can_enter": false,
      "integration_type": "api"
    },
    "charging_stations": [
      {
        "station_id": "charger-01",
        "distance_m": 15.0,
        "available": true,
        "compatible": true,
        "reserved": false,
        "connector_type": "wireless",
        "max_power_kw": 5.0
      }
    ],
    "infrastructure": {
      "wifi_access_points": 3,
      "ble_beacons_detected": 8,
      "uwb_anchors_detected": 4,
      "traffic_lights": [
        {
          "light_id": "tl-01",
          "state": "green",
          "time_to_change_sec": 15
        }
      ]
    },
    "surface": {
      "surface_type": "concrete",
      "condition": "dry",
      "slope_deg": 0.0,
      "friction_coefficient": 0.7,
      "bump_detected": false,
      "edge_detected": false
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `doors` | array | Door/gate interaction status |
| `elevators` | object | Elevator interaction and floor navigation |
| `charging_stations` | array | Nearby charging infrastructure |
| `infrastructure` | object | Environmental infrastructure detection |
| `surface` | object | Ground surface characteristics |

---

