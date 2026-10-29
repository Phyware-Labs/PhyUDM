# 18. Multi-Agent Coordination Domain

Fleet coordination, swarm behavior, and multi-robot collaboration.

```json
{
  "coordination": {
    "fleet_role": "follower",
    "formation": {
      "formation_id": "form-001",
      "formation_type": "line",
      "position_in_formation": 3,
      "formation_size": 5,
      "leader_id": "robot-001",
      "formation_integrity": 0.95,
      "spacing_error_m": 0.1
    },
    "neighbors": [
      {
        "robot_id": "robot-002",
        "distance_m": 2.5,
        "bearing_deg": -45.0,
        "relative_velocity_mps": 0.0,
        "communication_quality": 0.98,
        "last_heartbeat": "2026-01-02T10:34:58Z"
      }
    ],
    "shared_resources": {
      "pending_requests": 1,
      "held_resources": ["charging_station_3"],
      "waiting_for": ["aisle_7_access"]
    },
    "task_allocation": {
      "allocator": "market_based",
      "current_bid": 85.0,
      "competing_robots": 3,
      "allocation_status": "awarded"
    },
    "swarm": {
      "swarm_id": "swarm-001",
      "swarm_size": 50,
      "consensus_value": 0.87,
      "behavior_mode": "disperse",
      "local_density": 3.2,
      "separation_m": 0.5,
      "alignment_deg": 12.0,
      "cohesion_score": 0.91
    },
    "traffic": {
      "lane_id": "lane-A7",
      "traffic_direction": "north",
      "yielding_to": null,
      "right_of_way": true,
      "intersection_id": null,
      "deadlock_detected": false
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `fleet_role` | string | Role in fleet: `leader`, `follower`, `independent` |
| `formation` | object | Formation status for convoy/swarm operations |
| `neighbors` | array | Nearby robots with communication status |
| `shared_resources` | object | Resource allocation and deadlock avoidance |
| `task_allocation` | object | Multi-robot task allocation status |
| `swarm` | object | Swarm robotics behavior parameters |
| `traffic` | object | Multi-robot traffic management |

---

