# udm-eventstore-phycloud

[![Crate](https://img.shields.io/crates/v/udm-eventstore-phycloud.svg)](https://crates.io/crates/udm-eventstore-phycloud)
[![Docs](https://docs.rs/udm-eventstore-phycloud/badge.svg)](https://docs.rs/udm-eventstore-phycloud)

Reference deployment [`UdmEventStore`](../udm-eventstore/) adapter for
**PhyCloud** — the multi-tenant, Postgres-backed telemetry store that
ships as the sole commercially-supported deployment path for the OSS
UDM analysis tooling.

## Status — v0.0.3 (read path implemented)

This crate implements the `UdmEventStore` **read** contract against a live
PhyCloud deployment over its REST API:

- `query_events` → `GET /api/v1/events` with the indexable filters
  (`source_id`, `source_type`, `event_type`, `session_id`, capture-time range)
  pushed down as query-string params; any remaining `Predicate` terms are
  evaluated client-side over the fetched `include=event_data` payloads.
- `get_event` → `GET /api/v1/events/{event_id}`.
- `timeline` → time-ordered `query_events` for a single source.
- `aggregate` → `count` / `sum` / `avg` / `min` / `max` grouped by
  JSON-Pointer field paths, computed client-side over the fetched set.

`capabilities()` advertises aggregation, full-text `contains`, and ordered
streaming as supported.

### Session scoping

A single PhyCloud deployment can hold many missions side by side. Set
`PhyCloudConfig::session_id` (or pass `session_id=<mission>` in a `phycloud://`
`--store` URL) to constrain every query to one mission. For a local dev
deployment on plain HTTP, add `insecure=true` to the store URL so
`phycloud://localhost:8000?insecure=true&token=…&session_id=…` targets
`http://localhost:8000`.

### Not yet implemented

- **Write path** (ingestion goes through PhyCloud's REST API / the PhyTrace
  SDK directly, not this adapter).
- Native server-side aggregation (PhyCloud exposes no aggregate endpoint yet;
  the adapter emulates it client-side).
- Provenance-chain hydration in `get_event` (`include_provenance` is accepted
  but not yet round-tripped).

## License

Apache-2.0.
