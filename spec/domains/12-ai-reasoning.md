# 12. AI/Reasoning Domain

AI model decisions, confidence, and explainability.

```json
{
  "ai": {
    "models": [
      {
        "model_id": "nav-planner-v3",
        "model_type": "path_planning",
        "model_version": "3.2.1",
        "status": "active",
        "inference_time_ms": 12.5,
        "last_inference": "2026-01-02T10:34:58Z"
      },
      {
        "model_id": "object-detector-v2",
        "model_type": "perception",
        "model_version": "2.1.0",
        "status": "active",
        "inference_time_ms": 33.0,
        "fps": 30
      }
    ],
    "decisions": [
      {
        "decision_id": "dec-001",
        "timestamp": "2026-01-02T10:34:58Z",
        "model_id": "nav-planner-v3",
        "decision_type": "path_selection",
        "decision": "path_b",
        "confidence": 0.92,
        "alternatives": [
          {"option": "path_a", "score": 0.85},
          {"option": "path_c", "score": 0.78}
        ],
        "factors": [
          {"factor": "distance", "weight": 0.4, "value": 0.9},
          {"factor": "congestion", "weight": 0.3, "value": 0.95},
          {"factor": "battery", "weight": 0.3, "value": 0.88}
        ],
        "human_override": false
      }
    ],
    "anomalies": [
      {
        "anomaly_id": "anom-001",
        "timestamp": "2026-01-02T10:30:00Z",
        "anomaly_type": "behavior_deviation",
        "severity": "low",
        "description": "Unusual dwell time at location",
        "confidence": 0.75
      }
    ]
  }
}
```

---

