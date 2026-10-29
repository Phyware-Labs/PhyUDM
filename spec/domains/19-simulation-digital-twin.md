# 19. Simulation/Digital Twin Domain

Simulation parameters and digital twin synchronization.

```json
{
  "simulation": {
    "is_simulated": true,
    "simulator": {
      "name": "gazebo",
      "version": "harmonic",
      "physics_engine": "bullet",
      "real_time_factor": 1.0,
      "step_size_ms": 1.0
    },
    "scenario": {
      "scenario_id": "scen-warehouse-001",
      "scenario_name": "peak_load_test",
      "scenario_version": "2.1",
      "randomization_seed": 42,
      "parameters": {
        "human_density": "high",
        "obstacle_frequency": "medium",
        "lighting_condition": "normal"
      }
    },
    "digital_twin": {
      "paired_physical_id": "robot-001-physical",
      "sync_status": "synchronized",
      "sync_latency_ms": 50,
      "divergence_detected": false,
      "last_sync": "2026-01-02T10:34:58Z"
    },
    "fidelity": {
      "sensor_noise_enabled": true,
      "physics_fidelity": "high",
      "perception_fidelity": "medium",
      "communication_delay_simulated": true
    },
    "test_oracle": {
      "expected_behavior": "reach_goal",
      "pass_criteria": {
        "max_time_sec": 60,
        "min_safety_score": 90
      },
      "assertions_passed": 12,
      "assertions_failed": 0
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `is_simulated` | boolean | True if data from simulation |
| `simulator` | object | Simulator identification and configuration |
| `scenario` | object | Test scenario information |
| `digital_twin` | object | Physical/digital twin pairing status |
| `fidelity` | object | Simulation fidelity settings |
| `test_oracle` | object | Automated test validation status |

---

