//! The [`UdmEventStore`] trait that every UDM analysis backend implements.

use async_trait::async_trait;
use futures::stream::BoxStream;

use crate::event::{EventId, SourceId, UdmEvent};
use crate::query::{
    AggregateQuery, AggregateResult, EventPage, EventQuery, GetEventOptions, StoreCapabilities,
    TimeRange,
};
use crate::Result;

/// Boxed event stream returned by [`UdmEventStore::timeline`].
///
/// The `'a` lifetime is the borrow of the adapter — tools that need to
/// outlive the borrow should `Box::pin` an owned stream.
pub type EventStream<'a> = BoxStream<'a, Result<UdmEvent>>;

/// The adapter contract consumed by every UDM analysis tool.
///
/// Adapters MUST be `Send + Sync` so they can be shared across the
/// MCP server's tokio worker threads. They MAY be cloned freely (e.g.
/// behind an [`Arc`](std::sync::Arc)) — clone semantics are the
/// adapter's choice.
#[async_trait]
pub trait UdmEventStore: Send + Sync {
    /// Fetch one event by id.
    ///
    /// Returns `Ok(None)` when the id is well-formed but no matching
    /// event exists; returns [`Error::NotFound`](crate::Error::NotFound)
    /// only when the backend definitively rejects the id (e.g. wrong
    /// tenant).
    async fn get_event(&self, id: &EventId, opts: &GetEventOptions) -> Result<Option<UdmEvent>>;

    /// Structured paginated query.
    async fn query_events(&self, q: &EventQuery) -> Result<EventPage>;

    /// Time-ordered event stream for a single source over `range`.
    ///
    /// Events are emitted in ascending `captured_at` order. The stream
    /// completes when the backend has exhausted the range.
    async fn timeline(&self, source_id: &SourceId, range: TimeRange) -> Result<EventStream<'_>>;

    /// Aggregate `agg.field` by `agg.group_by` over events matching
    /// `agg.predicate` and `agg.time_range`.
    ///
    /// Adapters that don't implement aggregation natively MUST return
    /// [`Error::Unsupported`](crate::Error::Unsupported) instead of
    /// silently emulating.
    async fn aggregate(&self, agg: &AggregateQuery) -> Result<AggregateResult>;

    /// Backend capabilities — what this adapter actually supports.
    fn capabilities(&self) -> &StoreCapabilities;
}
