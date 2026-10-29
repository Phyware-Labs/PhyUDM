# 5. Operational State Domain

High-level operational mode and task status.

```json
{
  "operational": {
    "mode": "autonomous",
    "state": "executing_task",
    "sub_state": "navigating_to_goal",
    "availability": "available",
    "enabled": true,
    "e_stop_active": false,
    "safety_mode": "normal",
    "uptime_sec": 28800,
    "task": {
      "task_id": "task-2026-0001234",
      "task_type": "pick_and_place",
      "task_state": "in_progress",
      "task_priority": 5,
      "task_started_at": "2026-01-02T10:30:00Z",
      "task_eta": "2026-01-02T10:35:00Z",
      "task_progress_pct": 65.0,
      "task_source": "wms-integration",
      "task_payload": {
        "pick_location": "A7-15-3",
        "drop_location": "staging-1"
      }
    },
    "queue": {
      "queued_tasks": 3,
      "next_task_id": "task-2026-0001235"
    },
    "errors": [
      {
        "error_code": "NAV-001",
        "error_message": "Path blocked by obstacle",
        "error_severity": "warning",
        "error_timestamp": "2026-01-02T10:32:15Z"
      }
    ]
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `mode` | string | Operational mode (see Modes) |
| `state` | string | Current state (see States) |
| `sub_state` | string | Detailed sub-state |
| `availability` | string | Availability for new tasks |
| `enabled` | boolean | System enabled/disabled |
| `e_stop_active` | boolean | Emergency stop engaged |
| `safety_mode` | string | Current safety mode |
| `uptime_sec` | integer | System uptime (seconds) |
| `task` | object | Current task details |
| `queue` | object | Task queue status |
| `errors` | array | Active errors/warnings |

**Modes:** `autonomous`, `manual`, `teleoperated`, `semi_autonomous`, `collaborative`, `learning`, `maintenance`, `emergency`, `idle`, `standby`, `off`

**States:** `idle`, `executing_task`, `waiting`, `charging`, `error`, `maintenance`, `emergency_stop`, `initializing`, `shutting_down`, `paused`

---

