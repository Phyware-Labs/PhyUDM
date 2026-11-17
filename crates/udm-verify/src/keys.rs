//! Verification keys and the keyring events are checked against.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use p256::ecdsa::signature::Verifier as _;
use p256::ecdsa::{Signature, VerifyingKey};
use p256::pkcs8::{DecodePublicKey, EncodePublicKey};
use sha2::{Digest, Sha256};

use crate::hex;

/// `algorithm` value of ECDSA P-256 / SHA-256 signatures.
pub const ES256: &str = "es256";
/// `algorithm` value of HMAC-SHA256 signatures (development only).
pub const HMAC_SHA256: &str = "hmac-sha256";

/// Why a key could not be loaded.
#[derive(Debug, thiserror::Error)]
pub enum KeyError {
    /// The PEM or DER is not a P-256 public key.
    #[error("not a P-256 public key: {0}")]
    NotP256(String),
}

/// An ES256 (ECDSA P-256) public key.
#[derive(Debug, Clone)]
pub struct Es256Key {
    key: VerifyingKey,
    key_id: String,
}

impl Es256Key {
    /// From a PEM `PUBLIC KEY` (SubjectPublicKeyInfo).
    pub fn from_pem(pem: &str) -> Result<Self, KeyError> {
        let key = VerifyingKey::from_public_key_pem(pem.trim())
            .map_err(|e| KeyError::NotP256(e.to_string()))?;
        Self::from_key(key)
    }

    /// From DER SubjectPublicKeyInfo bytes.
    pub fn from_der(der: &[u8]) -> Result<Self, KeyError> {
        let key =
            VerifyingKey::from_public_key_der(der).map_err(|e| KeyError::NotP256(e.to_string()))?;
        Self::from_key(key)
    }

    fn from_key(key: VerifyingKey) -> Result<Self, KeyError> {
        let der = key
            .to_public_key_der()
            .map_err(|e| KeyError::NotP256(e.to_string()))?;
        let key_id = hex::encode(&Sha256::digest(der.as_bytes()));
        Ok(Self { key, key_id })
    }

    /// The key's id: lowercase hex SHA-256 of its DER SubjectPublicKeyInfo.
    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    /// Whether `signature_hex` (64-byte `r ‖ s`, either `s` or `n − s`) is a
    /// valid signature of `message`.
    pub fn verify(&self, message: &[u8], signature_hex: &str) -> bool {
        let Some(bytes) = hex::decode(signature_hex) else {
            return false;
        };
        let Ok(sig) = Signature::from_slice(&bytes) else {
            return false;
        };
        let sig = sig.normalize_s().unwrap_or(sig);
        self.key.verify(message, &sig).is_ok()
    }
}

/// The secret or public half of a key.
#[derive(Clone)]
pub enum KeyMaterial {
    /// An ES256 public key.
    Es256(Es256Key),
    /// An HMAC-SHA256 shared secret.
    Hmac(Vec<u8>),
}

impl std::fmt::Debug for KeyMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Es256(k) => f.debug_tuple("Es256").field(&k.key_id).finish(),
            Self::Hmac(_) => f.write_str("Hmac(<secret>)"),
        }
    }
}

impl KeyMaterial {
    /// The `algorithm` value signatures by this key carry.
    pub fn algorithm(&self) -> &'static str {
        match self {
            Self::Es256(_) => ES256,
            Self::Hmac(_) => HMAC_SHA256,
        }
    }

    /// Whether `signature_hex` is this key's signature of `message`.
    pub fn verify(&self, message: &[u8], signature_hex: &str) -> bool {
        match self {
            Self::Es256(key) => key.verify(message, signature_hex),
            Self::Hmac(secret) => {
                let (Some(sig), Ok(mut mac)) = (
                    hex::decode(signature_hex),
                    Hmac::<Sha256>::new_from_slice(secret),
                ) else {
                    return false;
                };
                mac.update(message);
                mac.verify_slice(&sig).is_ok()
            }
        }
    }
}

/// A key and what is known about it.
#[derive(Debug, Clone)]
pub struct KeyEntry {
    /// The key itself.
    pub material: KeyMaterial,
    /// The source the key belongs to; when set, events from any other
    /// source signed by this key are rejected.
    pub source_id: Option<String>,
    /// How the key is held, e.g. `hardware` (bound to a TPM) or `software`.
    pub binding: Option<String>,
    /// When a rotation replaced the key; events captured from then on are
    /// not accepted.
    pub retired_at: Option<DateTime<Utc>>,
    /// When the key was revoked; events captured from then on are not
    /// accepted.
    pub revoked_at: Option<DateTime<Utc>>,
}

impl KeyEntry {
    /// An entry with no source binding or lifecycle dates.
    pub fn new(material: KeyMaterial) -> Self {
        Self {
            material,
            source_id: None,
            binding: None,
            retired_at: None,
            revoked_at: None,
        }
    }
}

/// Keys by `key_id`.
#[derive(Debug, Clone, Default)]
pub struct Keyring {
    keys: BTreeMap<String, KeyEntry>,
}

impl Keyring {
    /// An empty keyring.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an ES256 key under its own key id; returns that id.
    pub fn add_es256(&mut self, key: Es256Key) -> String {
        let id = key.key_id().to_string();
        self.keys
            .insert(id.clone(), KeyEntry::new(KeyMaterial::Es256(key)));
        id
    }

    /// Add an HMAC secret under the key id events name it by.
    pub fn add_hmac(&mut self, key_id: impl Into<String>, secret: impl Into<Vec<u8>>) {
        self.keys.insert(
            key_id.into(),
            KeyEntry::new(KeyMaterial::Hmac(secret.into())),
        );
    }

    /// Add or replace an entry.
    pub fn insert(&mut self, key_id: impl Into<String>, entry: KeyEntry) {
        self.keys.insert(key_id.into(), entry);
    }

    /// The entry for `key_id`.
    pub fn get(&self, key_id: &str) -> Option<&KeyEntry> {
        self.keys.get(key_id)
    }

    /// Every entry, by key id.
    pub fn entries(&self) -> impl Iterator<Item = (&String, &KeyEntry)> {
        self.keys.iter()
    }

    /// Whether the keyring holds `key_id`.
    pub fn contains(&self, key_id: &str) -> bool {
        self.keys.contains_key(key_id)
    }

    /// Number of keys.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Whether the keyring is empty.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use p256::ecdsa::signature::Signer as _;
    use p256::ecdsa::SigningKey;
    use p256::pkcs8::LineEnding;

    #[test]
    fn es256_key_id_and_both_s_forms_verify() {
        let signing = SigningKey::from_slice(&[7u8; 32]).unwrap();
        let pem = VerifyingKey::from(&signing)
            .to_public_key_pem(LineEnding::LF)
            .unwrap();
        let key = Es256Key::from_pem(&pem).unwrap();
        assert_eq!(key.key_id().len(), 64);
        let sig: Signature = signing.sign(b"hello");
        assert!(key.verify(b"hello", &hex::encode(&sig.to_bytes())));
        // The high-s twin of a valid signature is accepted too.
        let (r, s) = sig.split_scalars();
        let high = Signature::from_scalars(r, -*s).unwrap();
        assert!(key.verify(b"hello", &hex::encode(&high.to_bytes())));
        assert!(!key.verify(b"hellO", &hex::encode(&sig.to_bytes())));
        assert!(!key.verify(b"hello", "zz"));
        assert!(Es256Key::from_pem("nope").is_err());
    }

    #[test]
    fn hmac_verifies_in_constant_time_api() {
        let k = KeyMaterial::Hmac(b"secret".to_vec());
        let mut mac = Hmac::<Sha256>::new_from_slice(b"secret").unwrap();
        mac.update(b"m");
        let sig = hex::encode(&mac.finalize().into_bytes());
        assert!(k.verify(b"m", &sig));
        assert!(!k.verify(b"n", &sig));
    }
}
