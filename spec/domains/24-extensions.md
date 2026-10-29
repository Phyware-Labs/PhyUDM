# 24. Extensions Domain

Vendor-specific and custom data.

```json
{
  "extensions": {
    "vendor": {
      "vendor_id": "locus_robotics",
      "vendor_schema_version": "2.1.0",
      "data": {
        "locus_task_priority_score": 85,
        "locus_picker_affinity": "zone-a",
        "locus_custom_metric": 123.45
      }
    },
    "platform": {
      "platform_id": "ros2_humble",
      "data": {
        "tf_buffer_size": 1000,
        "costmap_update_frequency": 5.0,
        "custom_param": "value"
      }
    },
    "customer": {
      "customer_id": "acme-corp",
      "data": {
        "shift_assignment": "A",
        "cost_center": "logistics-east",
        "custom_tag": "priority-picker"
      }
    },
    "raw": {
      "format": "ros2_odom",
      "encoding": "json",
      "data": {
        "header": {"stamp": {"sec": 1735820100, "nanosec": 500000000}},
        "pose": {"pose": {"position": {"x": 15.234, "y": 8.567, "z": 0}}},
        "twist": {"twist": {"linear": {"x": 1.2}, "angular": {"z": 0.05}}}
      }
    }
  }
}
```

| Namespace | Purpose |
|-----------|---------|
| `extensions.vendor` | Manufacturer/platform-specific data |
| `extensions.platform` | Middleware/framework-specific data (ROS, custom) |
| `extensions.customer` | Customer/tenant-specific fields |
| `extensions.raw` | Preserved original message (for audit/debug) |
| `extensions.custom.*` | Arbitrary custom namespaces |

---

