//! Compliance-standard lookup table (mirror of `udm-cli::compliance`).
//!
//! Duplicated rather than imported so `udm-mcp` doesn't depend on the
//! CLI's clap surface.

use udm_eventstore::Predicate;

/// One supported compliance standard.
pub struct ComplianceStandard {
    pub id: &'static str,
    pub name: &'static str,
    pub event_types: &'static [&'static str],
    pub notes: &'static str,
}

pub const STANDARDS: &[&ComplianceStandard] =
    &[&ISO_TS_15066, &ISO_13482, &ANSI_RIA_R15_06, &ISO_3691_4];

const ISO_TS_15066: ComplianceStandard = ComplianceStandard {
    id: "iso-ts-15066",
    name: "ISO/TS 15066 — Collaborative robots, biomechanical limits",
    event_types: &[
        "safety_violation",
        "emergency_stop",
        "task_started",
        "task_completed",
    ],
    notes: "Inspect every collaborative-mode transition and contact event; cross-check against \
            biomechanical force/pressure limits (Annex A).",
};

const ISO_13482: ComplianceStandard = ComplianceStandard {
    id: "iso-13482",
    name: "ISO 13482 — Safety requirements for personal-care robots",
    event_types: &[
        "safety_violation",
        "emergency_stop",
        "system_startup",
        "system_shutdown",
    ],
    notes: "Operational state + safety events bearing on protective stop behaviour and \
            speed-limit compliance.",
};

const ANSI_RIA_R15_06: ComplianceStandard = ComplianceStandard {
    id: "ansi-ria-r15.06",
    name: "ANSI/RIA R15.06 — Industrial robot safety",
    event_types: &["safety_violation", "emergency_stop", "task_failed"],
    notes: "Safety-system audit trail and incident response evidence.",
};

const ISO_3691_4: ComplianceStandard = ComplianceStandard {
    id: "iso-3691-4",
    name: "ISO 3691-4 — Industrial trucks (AGVs), safety",
    event_types: &[
        "safety_violation",
        "emergency_stop",
        "navigation.path_blocked",
    ],
    notes: "Driverless industrial-truck safety: zone breaches, e-stop traceability, blocked-path \
            response.",
};

pub fn lookup(id: &str) -> Option<&'static ComplianceStandard> {
    let id = id.to_ascii_lowercase();
    STANDARDS.iter().copied().find(|s| s.id == id)
}

pub fn audit_predicate(standard: &ComplianceStandard) -> Predicate {
    Predicate::In {
        field: "event_type".to_owned(),
        values: standard
            .event_types
            .iter()
            .map(|s| serde_json::Value::String((*s).to_owned()))
            .collect(),
    }
}
