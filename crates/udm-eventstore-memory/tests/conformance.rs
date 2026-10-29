//! End-to-end conformance check: hydrate the memory adapter from the
//! canonical fixture and run the full UdmEventStore suite.

use udm_eventstore::conformance::{load_fixture_events, run_full_suite};
use udm_eventstore_memory::MemoryStore;

#[tokio::test]
async fn memory_store_passes_full_conformance_suite() {
    let store = MemoryStore::from_events(load_fixture_events());
    run_full_suite(&store)
        .await
        .expect("memory store must pass");
}
