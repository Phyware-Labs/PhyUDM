# UDM AI Cookbook

> **Status — v0.0.3 draft.** Articles are runnable end-to-end against the
> bundled `memory`-adapter datasets. The articles have not yet been
> verified by hand in interactive LLM clients (Claude Desktop / Cursor /
> Copilot CLI); the code paths themselves are exercised by `crates/udm-mcp/tests/mcp_integration.rs`.

The cookbook shows LLM agents **consuming and analysing** UDM telemetry
from a running or historical autonomous system. Producer-side concerns
(generating UDM payloads) belong to the producer SDKs and the
`udm template` CLI subcommand, not here.

## How the articles are organised

| Article | What you'll learn | Dataset |
|---|---|---|
| [1. Wire UDM-MCP into Claude Desktop / Cursor / Copilot CLI](./01-wire-into-client.md) | Three side-by-side setup recipes; ends with a real query against a seeded fixture. | [`wire-into-client.ndjson`](./datasets/wire-into-client.ndjson) (6 events) |
| [2. LLM-driven incident reconstruction](./02-incident-reconstruction.md) | Use `incident_reconstruction` + `correlate_events` to root-cause an e-stop. | [`incident-amr-014.ndjson`](./datasets/incident-amr-014.ndjson) (14 events) |
| [3. Compliance audit with an LLM agent](./03-compliance-audit.md) | Bulk evidence pull for ISO/TS 15066, grouped + cited. | [`compliance-iso-ts-15066-q1.ndjson`](./datasets/compliance-iso-ts-15066-q1.ndjson) (195 events) |
| [4. Fleet health Q&A — live telemetry as a conversation](./04-fleet-health-qa.md) | Operator-style Q&A across a small fleet using `query_events` + `aggregate` + `timeline`. | [`fleet-health-warehouse-east.ndjson`](./datasets/fleet-health-warehouse-east.ndjson) (242 events) |

## Format conventions

Every article in this cookbook follows the same template:

1. **Scenario** — one paragraph describing what the agent is being asked
   to do, and why.
2. **Setup** — exactly the commands needed to point `udm-mcp` (or the
   `udm` CLI) at the seeded dataset shipped with the article.
3. **Walk-through** — the full tool-call trace: each MCP tool name +
   arguments + a truncated result snippet. No "the agent then figures
   it out" gaps.
4. **Final agent output** — the prose / table the LLM produces from the
   tool results.
5. **Try it yourself** — a copy-pasteable prompt that exercises the same
   path against the same dataset.
6. *(Optional)* **Going to production with PhyCloud** — what changes
   when you swap the `memory:` URL for `phycloud://`.

## Backend story

Every cookbook article runs end-to-end with **zero commercial
dependencies** by pointing `udm-mcp` (or `udm`) at the seeded
`docs/ai-cookbook/datasets/<article>.ndjson` fixture via the
`memory:///` URL scheme. The production path (PhyCloud) is shown as an
**optional appendix** in each article; the primary walk-through never
requires standing up PhyCloud.

In this positioning, the `memory` adapter
is the demos / reproducibility tier; PhyCloud is the sole reference
deployment adapter; third-party adapters are explicitly supported via
the `UdmEventStore` trait but not shipped in the OSS.

## Regenerating the datasets

```bash
python3 docs/ai-cookbook/build_datasets.py
```

The generator is deterministic (seeded RNG) so the datasets reproduce
bit-for-bit on any machine. Every event in every dataset validates
against `schemas/v0.0.3/event.schema.json`:

```bash
make validate-fixture ARGS="docs/ai-cookbook/datasets/wire-into-client.ndjson"
```

## License

Apache-2.0 (same as the rest of the repository).
