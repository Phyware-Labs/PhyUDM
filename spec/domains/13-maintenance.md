# 13. Maintenance Domain

Health, diagnostics, and maintenance tracking.

```json
{
  "maintenance": {
    "health_score": 92.0,
    "diagnostics": [
      {
        "component": "motor_left",
        "status": "ok",
        "health_pct": 95.0,
        "last_checked": "2026-01-02T08:00:00Z",
        "predicted_failure": null
      },
      {
        "component": "battery_pack",
        "status": "warning",
        "health_pct": 78.0,
        "last_checked": "2026-01-02T08:00:00Z",
        "predicted_failure": "2026-03-15",
        "recommendation": "Schedule battery replacement"
      }
    ],
    "maintenance_due": [
      {
        "maintenance_type": "wheel_inspection",
        "due_date": "2026-01-15",
        "due_hours": 50,
        "priority": "medium"
      }
    ],
    "maintenance_history": [
      {
        "maintenance_id": "maint-001",
        "maintenance_type": "software_update",
        "performed_at": "2026-01-01T06:00:00Z",
        "performed_by": "tech-042",
        "notes": "Updated navigation stack to v3.2.1"
      }
    ],
    "operating_hours": 2450.5,
    "operating_hours_since_maintenance": 150.5
  }
}
```

---

