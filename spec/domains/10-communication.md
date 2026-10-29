# 10. Communication Domain

Network, fleet, and integration status.

```json
{
  "communication": {
    "network": {
      "connected": true,
      "connection_type": "wifi",
      "ssid": "warehouse-iot",
      "signal_strength_dbm": -65,
      "ip_address": "192.168.1.101",
      "mac_address": "AA:BB:CC:DD:EE:FF",
      "latency_ms": 15,
      "bandwidth_mbps": 50.0,
      "packet_loss_pct": 0.1
    },
    "cellular": {
      "connected": false,
      "carrier": null,
      "signal_strength_dbm": null,
      "technology": null
    },
    "fleet": {
      "fleet_connected": true,
      "fleet_manager_id": "fleet-mgr-1",
      "last_heartbeat": "2026-01-02T10:34:55Z",
      "assigned_dispatcher": "dispatcher-east"
    },
    "integrations": [
      {
        "integration_id": "wms-sap",
        "status": "connected",
        "last_sync": "2026-01-02T10:34:50Z"
      }
    ],
    "vendor_extensions": {
      "agent_version": "1.2.0",
      "connected": true,
      "buffer_size": 150,
      "buffer_capacity": 10000,
      "last_upload": "2026-01-02T10:34:58Z",
      "upload_rate_eps": 10.0
    }
  }
}
```

---

