# udm-mcp

[![Crate](https://img.shields.io/crates/v/udm-mcp.svg)](https://crates.io/crates/udm-mcp)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](../../LICENSE)

`udm-mcp` — Model Context Protocol server that exposes UDM telemetry
analysis tools to any MCP-capable LLM agent (Claude Desktop, Cursor,
Copilot CLI, Continue, custom agent frameworks).

> ⭐ **The strategic moat of the PhyUDM epic.** UDM-MCP is the
> **LLM ↔ physical-world data interface**: it lets an analyst agent
> reach into a UDM-conforming telemetry store and reason about what an
> autonomous system did. Producer-side tooling (generating UDM payloads)
> lives in the `udm` CLI (`udm template`) and in producer SDKs — not
> here.

## Install + run

```bash
cargo install udm-mcp

# Then wire into your MCP-capable client (Claude Desktop, Cursor, …)
# by pointing it at the binary with the backend you want to analyse:

UDM_STORE=memory:///path/to/fleet.ndjson udm-mcp
```

The server speaks JSON-RPC 2.0 over stdio per the [Model Context
Protocol specification](https://modelcontextprotocol.io). Every log
line is written to stderr so it doesn't corrupt the stdout framing.

## Tools

### Data-plane (analysis — the primary surface)

| Tool | Purpose |
|---|---|
| `query_events` | Structured paginated search. Filter expressions: `field=value`, `field!=value`, `field in [a,b]`, `field contains text`, `field exists`. |
| `get_event` | Fetch one UDM event by id, optionally with provenance. |
| `timeline` | Time-ordered event stream for a single source across a window. |
| `correlate_events` | Find related events across domains around a seed event (incident root-cause). |
| `aggregate` | Fleet/group metrics (`count`/`sum`/`avg`/`min`/`max`). |
| `compliance_audit` | Every event bearing on a regulatory standard. Built-in: `iso-ts-15066`, `iso-13482`, `ansi-ria-r15.06`, `iso-3691-4`. |
| `incident_reconstruction` | Timeline-bracketed bundle around a moment. |

### Schema introspection (analyst-orientation helpers)

| Tool | Purpose |
|---|---|
| `explain_field` | Print spec metadata (type / description / enum) for any field path. |
| `list_event_types` | Discover what `event_type` values exist (with optional substring filter). |
| `validate_udm_event` | Sanity-check a raw payload against the canonical UDM event schema. |

## Backend selection

`--store URL` (or `UDM_STORE` env var) — same scheme as the `udm` CLI:

| Scheme | Example | Status |
|---|---|---|
| `memory:` | `memory:///path/fleet.ndjson` | ✅ fully functional (tests, demos, cookbook) |
| `phycloud:` | `phycloud://api.phycloud.example.com?token=KEY` | Read path implemented; no write path |

## Wiring into clients

### Claude Desktop

`~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "udm": {
      "command": "udm-mcp",
      "args": ["--store", "memory:///abs/path/fleet.ndjson"]
    }
  }
}
```

### Cursor

`~/.cursor/mcp.json` (or workspace `.cursor/mcp.json`):

```json
{
  "mcpServers": {
    "udm": {
      "command": "udm-mcp",
      "env": { "UDM_STORE": "memory:///abs/path/fleet.ndjson" }
    }
  }
}
```

### Copilot CLI

```bash
copilot mcp add udm --command udm-mcp --env UDM_STORE=memory:///abs/path/fleet.ndjson
```

> The PhyUDM repository ships seeded NDJSON datasets under
> [`docs/ai-cookbook/datasets/`](../../docs/ai-cookbook/datasets/) so
> you can try every tool without standing up a production backend.

## Try it yourself

Once wired up:

> _"Reconstruct the incident at 2026-06-07T19:00:05Z for amr-001 and explain the root cause."_

> _"Audit iso-ts-15066 compliance for fleet warehouse-a over Q1 2026; list every violation with provenance."_

> _"Summarize the last 24h of activity for source agv-002 — anomalies, mode changes, safety events."_

> _"Compare power-consumption patterns across source_type over the last week."_

## License

Apache-2.0.
