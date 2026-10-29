# 17. Human-Robot Interaction (HRI) Domain

Human interaction, collaboration, and social robotics data.

```json
{
  "hri": {
    "interaction_state": "collaborative_task",
    "humans_detected": 2,
    "tracked_humans": [
      {
        "human_id": "human-001",
        "tracking_confidence": 0.95,
        "distance_m": 1.5,
        "bearing_deg": 30.0,
        "velocity_mps": 0.5,
        "body_pose": "standing",
        "attention_on_robot": true,
        "gesture_detected": "wave",
        "gesture_confidence": 0.88,
        "face_detected": true,
        "emotion": "neutral",
        "ppe_detected": {
          "safety_vest": true,
          "hard_hat": true,
          "safety_glasses": false
        },
        "zone": "collaboration_zone"
      }
    ],
    "voice": {
      "voice_activity_detected": true,
      "speech_recognized": "pick up the box",
      "speech_confidence": 0.91,
      "speaker_id": "human-001",
      "language": "en-US",
      "command_parsed": {
        "intent": "pick_object",
        "object": "box",
        "confidence": 0.89
      }
    },
    "handover": {
      "in_progress": true,
      "direction": "robot_to_human",
      "object_id": "obj-123",
      "item_id": "TOOL-WR-15",
      "object_type": "tool",
      "object_class": "wrench",
      "human_id": "human-001",
      "phase": "extending",
      "ready_to_release": false
    },
    "social": {
      "greeting_given": true,
      "acknowledgment_pending": false,
      "user_satisfaction_score": 4.2,
      "interaction_duration_sec": 45.0
    },
    "safety_rating": {
      "iso_ts_15066_compliant": true,
      "current_operation_category": "power_and_force_limiting",
      "max_allowed_speed_mps": 0.25,
      "max_allowed_force_n": 140.0
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `interaction_state` | string | Current HRI state: `idle`, `approaching`, `collaborative_task`, `handover` |
| `tracked_humans` | array | Detected and tracked humans in vicinity |
| `tracked_humans[].ppe_detected` | object | Personal Protective Equipment detection |
| `voice` | object | Voice interaction and speech recognition |
| `handover` | object | Object handover state (robot ↔ human, uses `object_ref`) |
| `handover.object_id` | string | Session-stable ID of handover object |
| `handover.item_id` | string | External business identifier of handover object |
| `social` | object | Social interaction metrics (hospitality/service robots) |
| `safety_rating` | object | ISO/TS 15066 collaborative safety status |

---

