//! Checking one event's producer signature (spec `provenance.md`).

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::canonical::{canonical_json, leaf_payload, signing_payload};
use crate::hex;
use crate::keys::Keyring;
use crate::merkle::{leaf_hash, root_from_proof};

/// `type` of a batch statement.
pub const STATEMENT_TYPE: &str = "udm.merkle-batch.v1";

/// The same statement format under the name it had before PhyUDM defined
/// it; still accepted.
pub const LEGACY_STATEMENT_TYPE: &str = "phytrace.merkle-batch.v1";

/// Outcome of checking one event's signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SignatureStatus {
    /// A valid signature by a known key, current when the event was
    /// captured.
    Verified,
    /// The event carries no signature.
    Unsigned,
    /// The signature does not match the event, or (batch-signed) the
    /// event is not in the batch it claims.
    InvalidSignature,
    /// No key with the event's `key_id` is known (or it has none).
    UnknownKey,
    /// The key is known, but belongs to a different source.
    KeySourceMismatch,
    /// Signed by a key a rotation had replaced by the capture time.
    RetiredKey,
    /// Signed by a key revoked by the capture time.
    RevokedKey,
    /// The event's algorithm differs from the key's, or is unknown.
    UnverifiableAlgorithm,
}

impl SignatureStatus {
    /// The status as a string, as reports and manifests write it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Unsigned => "unsigned",
            Self::InvalidSignature => "invalid_signature",
            Self::UnknownKey => "unknown_key",
            Self::KeySourceMismatch => "key_source_mismatch",
            Self::RetiredKey => "retired_key",
            Self::RevokedKey => "revoked_key",
            Self::UnverifiableAlgorithm => "unverifiable_algorithm",
        }
    }
}

/// How an event is signed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SignatureMode {
    /// `provenance.signature` over the event itself.
    Single,
    /// `provenance.batch`: a signed Merkle batch statement and an
    /// inclusion proof.
    Batch,
}

/// Result of [`Verifier::check_event`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SignatureCheck {
    /// The outcome.
    pub status: SignatureStatus,
    /// `provenance.key_id`, if any.
    pub key_id: Option<String>,
    /// `provenance.algorithm`, if any.
    pub algorithm: Option<String>,
    /// How the event is signed; `None` when unsigned.
    pub mode: Option<SignatureMode>,
}

/// Checks events against a [`Keyring`].
///
/// Batch statements are cached, so checking every event of a batch costs
/// one signature verification.
#[derive(Debug)]
pub struct Verifier<'k> {
    keyring: &'k Keyring,
    statements: HashMap<[u8; 32], bool>,
}

impl<'k> Verifier<'k> {
    /// A verifier over `keyring`.
    pub fn new(keyring: &'k Keyring) -> Self {
        Self {
            keyring,
            statements: HashMap::new(),
        }
    }

    /// Check `event` (as the producer sent it) against the keyring.
    pub fn check_event(&mut self, event: &Value) -> SignatureCheck {
        let provenance = event.get("provenance").and_then(Value::as_object);
        let field = |name: &str| {
            provenance
                .and_then(|p| p.get(name))
                .and_then(Value::as_str)
                .map(str::to_string)
        };
        let key_id = field("key_id");
        let algorithm = field("algorithm");
        let batch = provenance
            .and_then(|p| p.get("batch"))
            .filter(|_| field("signature").is_none());
        let mode = match (batch, field("signature")) {
            (Some(_), _) => Some(SignatureMode::Batch),
            (None, Some(_)) => Some(SignatureMode::Single),
            (None, None) => None,
        };
        let done = |status| SignatureCheck {
            status,
            key_id: key_id.clone(),
            algorithm: algorithm.clone(),
            mode,
        };

        let signature = match batch {
            Some(b) => b
                .get("signature")
                .and_then(Value::as_str)
                .map(str::to_string),
            None => field("signature"),
        };
        let Some(signature) = signature else {
            return done(SignatureStatus::Unsigned);
        };
        let Some(entry) = key_id.as_deref().and_then(|id| self.keyring.get(id)) else {
            return done(SignatureStatus::UnknownKey);
        };
        if algorithm.as_deref() != Some(entry.material.algorithm()) {
            return done(SignatureStatus::UnverifiableAlgorithm);
        }
        if let Some(bound) = &entry.source_id {
            if event.get("source_id").and_then(Value::as_str) != Some(bound.as_str()) {
                return done(SignatureStatus::KeySourceMismatch);
            }
        }

        let valid = match batch {
            None => entry
                .material
                .verify(signing_payload(event).as_bytes(), &signature),
            Some(b) => {
                let statement = b.get("statement").unwrap_or(&Value::Null);
                let message = canonical_json(statement);
                let cache_key: [u8; 32] = {
                    let mut h = Sha256::new();
                    h.update(key_id.as_deref().unwrap_or("").as_bytes());
                    h.update([0]);
                    h.update(signature.as_bytes());
                    h.update([0]);
                    h.update(message.as_bytes());
                    h.finalize().into()
                };
                let signed = *self
                    .statements
                    .entry(cache_key)
                    .or_insert_with(|| entry.material.verify(message.as_bytes(), &signature));
                signed && in_batch(event, statement, b)
            }
        };
        if !valid {
            return done(SignatureStatus::InvalidSignature);
        }
        // Lifecycle is checked after the signature, so a forged event
        // learns nothing about which keys are revoked or retired.
        let captured = captured_at(event);
        let reached = |at: Option<DateTime<Utc>>| match (at, captured) {
            (Some(at), Some(captured)) => captured >= at,
            // A key with a revocation date and an event without a capture
            // time cannot be shown to predate the revocation.
            (Some(_), None) => true,
            (None, _) => false,
        };
        if reached(entry.revoked_at) {
            return done(SignatureStatus::RevokedKey);
        }
        if reached(entry.retired_at) {
            return done(SignatureStatus::RetiredKey);
        }
        done(SignatureStatus::Verified)
    }
}

/// The event's `captured_at`, if it has a valid one.
pub fn captured_at(event: &Value) -> Option<DateTime<Utc>> {
    event
        .get("captured_at")
        .and_then(Value::as_str)
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|t| t.with_timezone(&Utc))
}

/// Whether a batch-signed event is in the batch `statement` describes: its
/// inclusion proof leads to the statement's root, and its provenance
/// matches the statement (boot epoch, key, sequence range).
pub fn in_batch(event: &Value, statement: &Value, batch: &Value) -> bool {
    let provenance = event.get("provenance").unwrap_or(&Value::Null);
    let same = |field: &str| provenance.get(field) == statement.get(field);
    let typ = statement.get("type").and_then(Value::as_str);
    if !matches!(typ, Some(STATEMENT_TYPE | LEGACY_STATEMENT_TYPE))
        || !same("boot_epoch")
        || !same("key_id")
    {
        return false;
    }
    let sequence = provenance.get("sequence_num").and_then(Value::as_u64);
    let first = statement.get("first_sequence").and_then(Value::as_u64);
    let last = statement.get("last_sequence").and_then(Value::as_u64);
    match (sequence, first, last) {
        (Some(s), Some(a), Some(b)) if (a..=b).contains(&s) => {}
        _ => return false,
    }
    let (Some(index), Some(size), Some(root)) = (
        batch.get("leaf_index").and_then(Value::as_u64),
        statement.get("leaf_count").and_then(Value::as_u64),
        statement
            .get("root")
            .and_then(Value::as_str)
            .and_then(hex::hash),
    ) else {
        return false;
    };
    let Some(proof) = batch.get("proof").and_then(Value::as_array).and_then(|p| {
        p.iter()
            .map(|h| h.as_str().and_then(hex::hash))
            .collect::<Option<Vec<_>>>()
    }) else {
        return false;
    };
    let leaf = leaf_hash(leaf_payload(event).as_bytes());
    root_from_proof(index, size, &leaf, &proof) == Some(root)
}
