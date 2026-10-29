# 21. Audio Domain

Audio sensing, sound detection, and acoustic monitoring.

```json
{
  "audio": {
    "microphones": [
      {
        "mic_id": "mic_front",
        "status": "ok",
        "gain_db": 0.0,
        "sample_rate_hz": 48000,
        "channels": 4,
        "type": "array"
      }
    ],
    "speakers": [
      {
        "speaker_id": "speaker_main",
        "status": "ok",
        "volume_pct": 70.0,
        "playing": false,
        "current_audio": null
      }
    ],
    "sound_detection": {
      "ambient_level_db": 55.0,
      "peak_level_db": 72.0,
      "events": [
        {
          "event_id": "snd-001",
          "timestamp": "2026-01-02T10:34:55Z",
          "type": "human_speech",
          "confidence": 0.88,
          "direction_deg": 45.0,
          "distance_estimate_m": 3.0,
          "duration_ms": 1500
        },
        {
          "event_id": "snd-002",
          "timestamp": "2026-01-02T10:34:57Z",
          "type": "machinery",
          "confidence": 0.95,
          "direction_deg": -90.0
        }
      ]
    },
    "acoustic_signature": {
      "self_noise_profile": "normal",
      "motor_noise_db": 45.0,
      "anomaly_detected": false,
      "anomaly_type": null
    },
    "alerts": {
      "alarm_detected": false,
      "alarm_type": null,
      "emergency_siren": false,
      "horn_honk": false
    },
    "voice_output": {
      "tts_enabled": true,
      "current_utterance": null,
      "language": "en-US",
      "voice_profile": "professional"
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `microphones` | array | Microphone hardware status |
| `speakers` | array | Speaker hardware status |
| `sound_detection` | object | Detected acoustic events |
| `sound_detection.events[].type` | string | Sound classification: `human_speech`, `machinery`, `alarm`, `impact`, `glass_break` |
| `acoustic_signature` | object | Robot's own noise profile and anomaly detection |
| `alerts` | object | Safety-relevant sound detections |
| `voice_output` | object | Text-to-speech status |

---

