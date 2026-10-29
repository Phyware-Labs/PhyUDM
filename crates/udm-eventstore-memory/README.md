# udm-eventstore-memory

[![Crate](https://img.shields.io/crates/v/udm-eventstore-memory.svg)](https://crates.io/crates/udm-eventstore-memory)
[![Docs](https://docs.rs/udm-eventstore-memory/badge.svg)](https://docs.rs/udm-eventstore-memory)

In-process `Vec<UdmEvent>` implementation of
[`UdmEventStore`](../udm-eventstore/). Designed for **testing, demos,
and cookbook reproducibility** — not a production deployment target.

For production, use
[`udm-eventstore-phycloud`](../udm-eventstore-phycloud) or implement
your own adapter against the `UdmEventStore` trait.

## Construction

```rust,no_run
# async fn run() -> udm_eventstore::Result<()> {
use udm_eventstore_memory::MemoryStore;

// From an explicit Vec<UdmEvent>:
use udm_eventstore::conformance::load_fixture_events;
let store = MemoryStore::from_events(load_fixture_events());

// From an NDJSON string:
let store = MemoryStore::from_ndjson_str("{\"udm_version\":\"0.0.3\", /*...*/}")?;

// From an NDJSON file:
let store = MemoryStore::from_ndjson_path("docs/ai-cookbook/datasets/incident.ndjson").await?;
# Ok(()) }
```

## Capabilities

The memory adapter advertises full support for every operation in the
`UdmEventStore` contract (`supports_aggregation`, `supports_full_text`,
`supports_ordered_streaming` all `true`). This makes it a useful
ceiling against which to test other adapters — anything that's not
implementable in the memory adapter probably shouldn't be in the
trait.

## License

Apache-2.0.
