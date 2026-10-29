# 23. Compliance/Certification Domain

Regulatory compliance status and certification tracking.

```json
{
  "compliance": {
    "certifications": [
      {
        "standard": "ISO_13482",
        "version": "2014",
        "status": "certified",
        "certificate_id": "CERT-2025-12345",
        "issued_date": "2025-06-15",
        "expiry_date": "2028-06-15",
        "certifying_body": "TÜV SÜD"
      },
      {
        "standard": "ISO_10218",
        "version": "2011",
        "status": "certified",
        "certificate_id": "CERT-2025-12346"
      },
      {
        "standard": "CE_MARKING",
        "status": "compliant",
        "declaration_id": "DOC-2025-001"
      }
    ],
    "functional_safety": {
      "safety_integrity_level": "SIL2",
      "performance_level": "PLd",
      "safety_controller_status": "ok",
      "safety_plc_firmware": "3.1.0",
      "watchdog_status": "ok",
      "last_safety_test": "2026-01-01T06:00:00Z"
    },
    "cybersecurity": {
      "encryption_enabled": true,
      "firmware_signed": true,
      "last_security_scan": "2026-01-01T00:00:00Z",
      "vulnerabilities_known": 0,
      "security_patch_level": "2025-12",
      "authentication_method": "mtls",
      "secure_boot_enabled": true
    },
    "data_privacy": {
      "gdpr_compliant": true,
      "data_retention_days": 90,
      "anonymization_enabled": true,
      "consent_status": "obtained",
      "pii_detected_in_session": false
    },
    "operational_compliance": {
      "operating_license_valid": true,
      "geofence_compliant": true,
      "speed_limit_compliant": true,
      "noise_limit_compliant": true,
      "emissions_compliant": true
    }
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `certifications` | array | Active certifications and standards compliance |
| `functional_safety` | object | IEC 61508/62443 functional safety status |
| `cybersecurity` | object | Cybersecurity posture and compliance |
| `data_privacy` | object | GDPR/privacy compliance status |
| `operational_compliance` | object | Runtime operational rule compliance |

---

