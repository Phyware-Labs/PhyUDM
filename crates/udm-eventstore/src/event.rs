//! Typed UDM event envelope shared by every adapter.
//!
//! The struct mirrors `schemas/v0.0.3/event.schema.json`:
//! - Required envelope fields (`udm_version`, `event_id`, `event_type`,
//!   `source_id`, `source_type`, `captured_at`).
//! - All optional envelope fields (`received_at`, `sent_at`,
//!   `sequence_num`, `session_id`, `sdk_version`).
//! - 23 optional domain slots collapsed into a single `domains` map so
//!   the type stays stable while new domains land in MINOR releases.
//! - Optional `provenance` block.
//! - Free-form `extensions` payload.
//!
//! Round-trips losslessly to/from the canonical JSON via
//! [`serde_json::to_value`] and [`serde_json::from_value`].

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Opaque event identifier (UUIDv7 recommended, but adapters MUST accept
/// any non-empty string so legacy backends remain usable).
pub type EventId = String;

/// Stable source identifier (robot, sensor, fleet manager, ...).
pub type SourceId = String;

/// The 23 structured UDM domain slots (snake_case keys matching the
/// canonical JSON Schema names).
///
/// Stored as a `BTreeMap` so iteration is deterministic; adapters that
/// project from a SQL or columnar backend can populate only the domains
/// the caller asked for.
pub type DomainMap = BTreeMap<String, serde_json::Value>;

/// Canonical UDM event payload.
///
/// Serializes as the flat JSON shape defined by
/// `schemas/v0.0.3/event.schema.json`: envelope fields at the top level,
/// each domain key inlined alongside.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UdmEvent {
    // ---------------------------------------------------------------
    // Required envelope fields.
    // ---------------------------------------------------------------
    /// Schema version this event conforms to (SemVer).
    pub udm_version: String,

    /// Globally unique event identifier; UUIDv7 recommended.
    pub event_id: EventId,

    /// Event classification (see canonical `EventType` enum).
    pub event_type: String,

    /// Stable identifier of the emitting source.
    pub source_id: SourceId,

    /// Source classification (see canonical `SourceType` enum).
    pub source_type: String,

    /// ISO-8601 timestamp when the data was captured at the source.
    pub captured_at: DateTime<Utc>,

    // ---------------------------------------------------------------
    // Optional envelope fields.
    // ---------------------------------------------------------------
    /// ISO-8601 timestamp when the ingest layer received the event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub received_at: Option<DateTime<Utc>>,

    /// ISO-8601 timestamp when the source sent the event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sent_at: Option<DateTime<Utc>>,

    /// Monotonic sequence number from the source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequence_num: Option<u64>,

    /// Session/run identifier for grouping events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    /// Optional emitting SDK version. The Rust SDK populates this;
    /// third-party emitters MAY omit it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sdk_version: Option<String>,

    // ---------------------------------------------------------------
    // Domain payloads + provenance + extensions.
    //
    // Domains are flattened so the JSON shape matches the canonical
    // schema, but on the Rust side they live in a single map for
    // ergonomic iteration.
    // ---------------------------------------------------------------
    /// Per-domain payload map (`"identity"`, `"location"`, …). Keys
    /// match the canonical domain identifiers in
    /// `schemas/v0.0.3/domains/`.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub domains: DomainMap,
}

impl UdmEvent {
    /// Convenience accessor: fetch a domain payload by canonical key.
    pub fn domain(&self, name: &str) -> Option<&serde_json::Value> {
        self.domains.get(name)
    }

    /// Convenience accessor: list the populated domain keys in
    /// canonical (BTreeMap) order.
    pub fn populated_domains(&self) -> impl Iterator<Item = &str> {
        self.domains.keys().map(String::as_str)
    }

    /// Lookup the `provenance` block as a typed [`Provenance`].
    pub fn provenance(&self) -> Option<Provenance> {
        self.domains
            .get("provenance")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

/// Event-level integrity / provenance metadata mirroring
/// `schemas/v0.0.3/event.schema.json` → `provenance`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Provenance {
    /// Signature value (HMAC-SHA256 or similar).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// Identifier of the signing key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_id: Option<String>,
    /// Signature algorithm (e.g. `"hmac-sha256"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<String>,
    /// Fields included in the signature.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signed_fields: Option<Vec<String>>,
    /// Timestamp when the signature was created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signed_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    fn sample() -> serde_json::Value {
        json!({
            "udm_version": "0.0.3",
            "event_id": "01940000-0000-7000-8000-000000000001",
            "event_type": "telemetry_periodic",
            "source_id": "amr-001",
            "source_type": "amr",
            "captured_at": "2026-06-07T19:00:00Z",
            "identity": {"source_id": "amr-001", "source_type": "amr"},
            "location": {"latitude": 32.7767, "longitude": -96.7970},
        })
    }

    #[test]
    fn roundtrips_via_serde() {
        let original = sample();
        let event: UdmEvent = serde_json::from_value(original.clone()).unwrap();
        assert_eq!(event.udm_version, "0.0.3");
        assert_eq!(event.event_id, "01940000-0000-7000-8000-000000000001");
        assert_eq!(event.source_id, "amr-001");
        assert!(event.domains.contains_key("identity"));
        assert!(event.domains.contains_key("location"));
        let serialized = serde_json::to_value(&event).unwrap();
        assert_eq!(serialized, original);
    }

    #[test]
    fn populated_domains_are_ordered() {
        let event: UdmEvent = serde_json::from_value(sample()).unwrap();
        let keys: Vec<&str> = event.populated_domains().collect();
        assert_eq!(keys, vec!["identity", "location"]);
    }
}
