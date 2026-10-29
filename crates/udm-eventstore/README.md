# udm-eventstore

[![Crate](https://img.shields.io/crates/v/udm-eventstore.svg)](https://crates.io/crates/udm-eventstore)
[![Docs](https://docs.rs/udm-eventstore/badge.svg)](https://docs.rs/udm-eventstore)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](../../LICENSE)

`UdmEventStore` — the adapter trait that lets UDM analysis tooling
(the `udm` CLI, the `udm-mcp` server) query any UDM-conforming
telemetry backend through a single contract.

## What's here

- `UdmEventStore` — the async trait every backend implements.
- Supporting types: `UdmEvent`, `EventQuery`, `Predicate`, `TimeRange`,
  `AggregateQuery`, `AggregateResult`, `StoreCapabilities`.
- `conformance::run_full_suite` — a reusable acceptance harness so
  community adapters can self-test against the canonical fixture.

## Why a trait?

UDM is a **vendor-neutral data model**, so the analysis tooling needs
to stay decoupled from any single backend's wire format or query
language. The trait lets:

- The OSS analysis layer (`udm` CLI + `udm-mcp` server) ship with a
  single, tightly-supported reference deployment adapter (PhyCloud,
  via `udm-eventstore-phycloud`) without precluding third-party
  adapters.
- Anyone who emits UDM today drop in their own adapter and immediately
  benefit from the entire LLM analysis surface — incident
  reconstruction, compliance audits, fleet Q&A — without forking the
  analysis tooling.
- Tests, demos, and cookbook articles run zero-dependency against the
  in-process `udm-eventstore-memory` adapter.

## Adapters shipped at v0.0.x

| Crate                          | Backend                                    | Purpose                                    |
|--------------------------------|--------------------------------------------|--------------------------------------------|
| `udm-eventstore-memory`        | In-process `Vec<UdmEvent>` from NDJSON     | Testing, demos, cookbook reproducibility   |
| `udm-eventstore-phycloud`      | PhyCloud (sole reference deployment path)  | Production — read path implemented; no write path |

No `file`, raw `postgres`, ClickHouse, or S3+Parquet adapters ship at
v0.1.0 — those are valid future / third-party work, but not part of
the OSS surface.

## Quick start (adapter author)

```rust,no_run
use async_trait::async_trait;
use udm_eventstore::{
    EventId, EventPage, EventQuery, EventStream, GetEventOptions, Result,
    SourceId, StoreCapabilities, TimeRange, UdmEvent, UdmEventStore,
    AggregateQuery, AggregateResult,
};

pub struct MyAdapter { /* ... */ }

#[async_trait]
impl UdmEventStore for MyAdapter {
    async fn get_event(&self, id: &EventId, opts: &GetEventOptions) -> Result<Option<UdmEvent>> {
        todo!()
    }
    async fn query_events(&self, q: &EventQuery) -> Result<EventPage> { todo!() }
    async fn timeline(&self, source_id: &SourceId, range: TimeRange) -> Result<EventStream<'_>> { todo!() }
    async fn aggregate(&self, agg: &AggregateQuery) -> Result<AggregateResult> { todo!() }
    fn capabilities(&self) -> &StoreCapabilities { todo!() }
}
```

Then validate behaviour against the conformance suite:

```rust,ignore
use udm_eventstore::conformance::{load_fixture_events, run_full_suite};

#[tokio::test]
async fn my_adapter_passes_conformance() {
    let adapter = MyAdapter::with_events(load_fixture_events());
    run_full_suite(&adapter).await.expect("must pass the suite");
}
```

## License

Apache-2.0.
