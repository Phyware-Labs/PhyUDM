# Provenance Metadata


For immutability and chain-of-custody requirements, events MAY include provenance metadata:

```json
{
  "provenance": {
    "capture_source": "udm_agent",
    "capture_version": "1.2.0",
    "capture_host": "robot-001",
    "ingest_node": "udm-ingest-east-1",
    "ingest_timestamp": "2026-01-02T10:35:00.345678Z",
    "hash": "sha256:abc123...",
    "previous_hash": "sha256:def456...",
    "signature": "ed25519:...",
    "chain_id": "chain-robot-001-2026-01-02"
  }
}
```

---

