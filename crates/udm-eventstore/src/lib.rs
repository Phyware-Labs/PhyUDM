//! UdmEventStore — the adapter trait that lets UDM analysis tooling
//! (the [`udm` CLI](https://github.com/Phyware-Labs/PhyUDM/tree/main/crates/udm-cli),
//! the [`udm-mcp` server](https://github.com/Phyware-Labs/PhyUDM/tree/main/crates/udm-mcp))
//! query any UDM-conforming telemetry backend through a single contract.
//!
//! # Design intent
//!
//! - **Backend-agnostic by construction.** Analysis tooling consumes this
//!   trait and never references a concrete backend type directly.
//! - **Read-only at v0.0.x.** UDM is fundamentally an emit-then-analyse
//!   data model; write paths belong to producer SDKs, not the analysis
//!   layer.
//! - **Capability-negotiated.** Tools call [`UdmEventStore::capabilities`]
//!   and degrade gracefully rather than the trait pretending every
//!   backend can do everything.
//! - **Streaming where it matters.** [`UdmEventStore::timeline`] returns a
//!   stream because incident-reconstruction queries can span 10⁵+ events;
//!   [`UdmEventStore::query_events`] is bounded-page because most analysis
//!   tools want paginated access.
//!
//! # Adapters shipped at v0.0.x
//!
//! | Crate                          | Backend                              | Purpose                                  |
//! |--------------------------------|--------------------------------------|------------------------------------------|
//! | `udm-eventstore-memory`        | In-process `Vec<UdmEvent>` from NDJSON | Testing, demos, cookbook reproducibility |
//! | `udm-eventstore-phycloud`      | PhyCloud (sole reference deployment) | Production. Read path implemented        |
//!
//! Third-party adapters are explicitly supported via the trait, but no
//! other adapters ship as part of the OSS at v0.1.0.
//!
//! # Conformance
//!
//! Any new adapter SHOULD pass the language-agnostic conformance helpers
//! in [`conformance`]. The reference implementation in
//! [`crate::conformance::run_full_suite`] checks every trait method
//! against a deterministic fixture set.

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod conformance;
pub mod error;
pub mod event;
pub mod query;
mod store;

pub use error::{Error, Result};
pub use event::{EventId, Provenance, SourceId, UdmEvent};
pub use query::{
    AggregateBucket, AggregateFn, AggregateQuery, AggregateResult, EventPage, EventQuery,
    GetEventOptions, OrderBy, Predicate, StoreCapabilities, TimeRange,
};
pub use store::{EventStream, UdmEventStore};
