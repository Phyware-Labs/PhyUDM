//! Error and Result types for the `UdmEventStore` contract.

use thiserror::Error;

/// Crate-wide `Result` alias.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// All error conditions surfaced by an [`UdmEventStore`](crate::UdmEventStore)
/// adapter.
///
/// Adapters MUST map their backend-specific errors into one of these
/// variants so analysis tooling can pattern-match the kind of failure
/// without depending on any concrete backend.
#[derive(Debug, Error)]
pub enum Error {
    /// The caller asked for something the backend does not support.
    ///
    /// Tools SHOULD interrogate
    /// [`UdmEventStore::capabilities`](crate::UdmEventStore::capabilities)
    /// before issuing a request and SHOULD treat `Unsupported` as
    /// permanent — retries will not change the outcome.
    #[error("unsupported operation: {0}")]
    Unsupported(String),

    /// The caller submitted an invalid query (bad time range, malformed
    /// predicate, …). This is a user error, not a backend failure.
    #[error("invalid query: {0}")]
    InvalidQuery(String),

    /// An event referenced by id was not found. Distinct from
    /// `Ok(None)` returns; reserved for cases where the backend
    /// definitively says "no" rather than "I looked and there was
    /// nothing matching".
    #[error("event not found: {0}")]
    NotFound(String),

    /// The backend is temporarily unavailable (network blip, retry
    /// budget exhausted, ...). Callers MAY retry with backoff.
    #[error("backend unavailable: {0}")]
    Unavailable(String),

    /// Authentication or authorization failed against the backend.
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// Serialization or deserialization of a payload failed.
    #[error("serde: {0}")]
    Serde(#[from] serde_json::Error),

    /// An I/O operation failed (file load, network, …).
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    /// Catch-all for backend-specific errors that do not map to any
    /// other variant. Adapters SHOULD prefer the more specific
    /// variants where possible.
    #[error("backend: {0}")]
    Backend(String),
}
