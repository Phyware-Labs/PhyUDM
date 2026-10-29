//! Compliance-standard → UDM-field mapping table consumed by the
//! `audit` subcommand (and re-used by the MCP server's
//! `compliance_audit` tool).
//!
//! Each standard maps to a list of `event_type` values that bear on
//! audit evidence. This is a first-pass mapping table; future PRs
//! will add per-field force/distance threshold rules.

use udm_eventstore::Predicate;

/// One supported compliance standard.
pub struct ComplianceStandard {
    /// Canonical identifier (lowercased, dash-separated).
    pub id: &'static str,
    /// Short human-readable name.
    pub name: &'static str,
    /// Event types that bear on this standard. Used as an `In`
    /// predicate over `event_type` to scope an audit query.
    pub event_types: &'static [&'static str],
    /// Free-form notes shown alongside the audit output.
    pub notes: &'static str,
}

/// All compliance standards the CLI / MCP server know about at v0.0.3.
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
    notes: "Look at every collaborative-mode transition and contact event; cross-check against \
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

/// Look up a standard by id (case-insensitive).
pub fn lookup(id: &str) -> Option<&'static ComplianceStandard> {
    let id = id.to_ascii_lowercase();
    STANDARDS.iter().copied().find(|s| s.id == id)
}

/// Build the predicate scoping an audit query for `standard`.
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
