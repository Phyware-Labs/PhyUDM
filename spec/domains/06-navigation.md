# 6. Navigation Domain

Path planning, localization, and navigation status.

```json
{
  "navigation": {
    "localization": {
      "status": "localized",
      "confidence": 0.98,
      "method": "amcl",
      "map_id": "warehouse-east-v3",
      "pose_covariance": [0.01, 0, 0, 0, 0.01, 0, 0, 0, 0.02]
    },
    "slam": {
      "active": false,
      "mode": "localization_only",
      "map_quality": 0.95,
      "loop_closures": 42,
      "keyframes": 1250,
      "map_size_mb": 125.5
    },
    "path": {
      "has_path": true,
      "path_length_m": 15.7,
      "path_eta_sec": 12.5,
      "waypoints_remaining": 3,
      "path_blocked": false,
      "replanning": false
    },
    "goal": {
      "goal_id": "goal-001",
      "goal_x_m": 25.0,
      "goal_y_m": 12.0,
      "goal_yaw_deg": 90.0,
      "distance_to_goal_m": 8.3,
      "goal_type": "pickup"
    },
    "obstacles": {
      "nearest_obstacle_m": 2.5,
      "obstacle_count": 3,
      "dynamic_obstacles": 1,
      "static_obstacles": 2
    },
    "costmap": {
      "inflation_radius_m": 0.5,
      "robot_radius_m": 0.3
    },
    "multi_floor": {
      "current_floor": 1,
      "target_floor": 2,
      "floor_transition_method": "elevator",
      "floor_transition_status": "awaiting_elevator",
      "floors_in_mission": [1, 2]
    },
    "semantic_map": {
      "current_region": "warehouse-zone-a",
      "region_type": "storage",
      "nearby_pois": [
        {"poi_id": "charger-01", "type": "charging_station", "distance_m": 5.2},
        {"poi_id": "door-A3", "type": "door", "distance_m": 8.1}
      ],
      "lane_id": "lane-north-7",
      "allowed_regions": ["zone-a", "zone-b", "corridor-1"],
      "restricted_regions": ["zone-c", "maintenance-area"]
    },
    "planner": {
      "global_planner": "navfn",
      "local_planner": "dwb",
      "behavior_tree": "navigate_to_pose.xml",
      "recovery_active": false,
      "recovery_behavior": null
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `localization.status` | string | Status: `not_localized`, `localizing`, `localized`, `lost` |
| `localization.method` | string | Method: `amcl`, `slam`, `gps`, `uwb`, `fusion` |
| `slam` | object | SLAM system status when active |
| `slam.mode` | string | Mode: `mapping`, `localization_only`, `mapping_and_localization` |
| `path` | object | Current path status |
| `goal` | object | Active navigation goal |
| `obstacles` | object | Detected obstacles summary |
| `costmap` | object | Costmap configuration |
| `multi_floor` | object | Multi-floor navigation status |
| `semantic_map` | object | Semantic/topological map context |
| `planner` | object | Active planner configuration |

---

