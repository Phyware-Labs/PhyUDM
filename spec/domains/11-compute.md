# 11. Compute Domain

Onboard compute and resource utilization.

```json
{
  "compute": {
    "cpu": {
      "usage_pct": 45.0,
      "temperature_c": 55.0,
      "frequency_mhz": 2400,
      "cores": 8
    },
    "memory": {
      "used_mb": 4096,
      "total_mb": 16384,
      "usage_pct": 25.0
    },
    "gpu": {
      "usage_pct": 30.0,
      "memory_used_mb": 2048,
      "memory_total_mb": 8192,
      "temperature_c": 60.0
    },
    "storage": {
      "used_gb": 50.0,
      "total_gb": 256.0,
      "usage_pct": 19.5
    },
    "processes": {
      "navigation_stack": {"status": "running", "cpu_pct": 15.0, "memory_mb": 512},
      "perception": {"status": "running", "cpu_pct": 20.0, "memory_mb": 1024},
      "planner": {"status": "running", "cpu_pct": 5.0, "memory_mb": 256}
    },
    "ros": {
      "ros_version": "humble",
      "node_count": 25,
      "topic_count": 150,
      "master_connected": true
    }
  }
}
```

---

