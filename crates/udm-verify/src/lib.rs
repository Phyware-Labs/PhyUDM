//! Open verifier for PhyUDM provenance.
//!
//! `udm-verify` checks UDM events and evidence packages without trusting,
//! or even reaching, the service that collected them:
//!
//! - **Event signatures** ([`event`]): the producer's signature over the
//!   event's [signing payload](canonical::signing_payload), single or
//!   Merkle-batched, ES256 or HMAC-SHA256 (spec `provenance.md`).
//! - **Ordering and replay** ([`sequence`]): each source's
//!   `(boot_counter, sequence_num)` stream, classified as in order, gap,
//!   restart, late, duplicate, reset or regression.
//! - **Evidence packages** ([`package`]): a receiver's signed manifest, its
//!   hash chain and Merkle root, and every event as received, with each
//!   device signature checked again (spec `evidence-package.md`).
//!
//! Trust is explicit: a key found inside a package is only used to check
//! signatures that the package's own manifest vouches for, and a manifest
//! counts as trusted only when its signing key is one the caller pinned.
//!
//! The crate does no I/O of its own (packages are read from any
//! [`std::io::BufRead`]), so it builds for WebAssembly and embeds in other
//! languages.

pub mod canonical;
pub mod event;
pub mod hex;
pub mod keys;
pub mod merkle;
pub mod package;
pub mod sequence;

pub use event::{SignatureCheck, SignatureMode, SignatureStatus, Verifier};
pub use keys::{Es256Key, KeyEntry, KeyMaterial, Keyring};
pub use package::{verify_package, PackageOptions, PackageReport, Verdict};
pub use sequence::{SequenceInfo, SequenceStatus, SequenceTracker};
