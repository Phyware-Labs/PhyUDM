# udm-cli

[![Crate](https://img.shields.io/crates/v/udm-cli.svg)](https://crates.io/crates/udm-cli)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](../../LICENSE)

`udm` — command-line tool for validating, exploring, and analysing UDM
telemetry. Self-contained binary; ships every JSON Schema artifact
embedded at build time so the validator works offline.

```
$ udm --help
udm — command-line tool for validating, exploring, and analysing UDM telemetry.

Usage: udm [OPTIONS] <COMMAND>

Commands:
  validate     Validate a JSON payload against a UDM schema version
  schema       Inspect UDM JSON Schema artifacts (show, diff)
  explain      Print the spec text for a field path (e.g. `safety/safety_state`)
  conformance  Run the embedded conformance suite against the canonical schema
  template     Print a skeleton UDM event for hand-editing
  query        Structured search over the event store
  get          Fetch a single event by id
  timeline     Stream the timeline for a single source
  correlate    Find related events across domains around a seed event
  audit        Run a compliance audit over a window
  aggregate    Compute group / fleet metrics
```

## Install

```bash
# From crates.io (once published):
cargo install udm-cli

# Or from a local checkout:
cargo install --path crates/udm-cli
```

## Audiences

The CLI serves two audiences with the same surface:

1. **Spec / schema authors and CI pipelines** — `validate`, `schema show/diff`,
   `explain`, `conformance run`, `template`.
2. **Operators and analyst LLM agents** (Claude Code, Copilot CLI, shell
   scripts) — `query`, `get`, `timeline`, `correlate`, `audit`, `aggregate`.
   Same `UdmEventStore` trait the MCP server uses, so anything that works
   in the CLI works in `udm-mcp` and vice versa.

## Backend selection (`--store` / `UDM_STORE`)

Analysis subcommands take a `--store` URL or `UDM_STORE` env var.
Supported schemes at v0.0.3:

| Scheme | Example | Notes |
|---|---|---|
| `memory:` | `memory:///abs/path/fleet.ndjson` | Loads an NDJSON file into RAM via the `udm-eventstore-memory` adapter. Testing / demos / cookbook. |
| `phycloud:` | `phycloud://api.phycloud.example.com?token=KEY` | Sole reference deployment adapter. Read path implemented (query, get, timeline, aggregate); no write path. |

## Worked examples

### Validate a payload

```bash
udm validate path/to/event.json --schema-version 0.0.3
```

JSON-Lines output by default; `--output pretty` for human consumption.

### Generate a skeleton for hand-editing

```bash
udm template \
  --source-type amr \
  --event-type telemetry_periodic \
  --domains identity,location,motion,power,safety \
  --output pretty
```

### Inspect a field

```bash
udm explain safety/safety_state
```

```json
{
  "path": "safety/safety_state",
  "schema_version": "0.0.3",
  "type": "string",
  "enum": ["normal", "warning", "protective_stop", "emergency_stop", "safety_interlock", "reduced_speed"]
}
```

### Query an event store

```bash
export UDM_STORE=memory:///path/to/fleet.ndjson
udm query --filter "event_type=safety_violation" --from 2026-06-07T00:00:00Z --to 2026-06-08T00:00:00Z
```

### Timeline for one robot

```bash
udm timeline amr-001 --from 2026-06-07T19:00:00Z --to 2026-06-07T19:01:00Z
```

### Correlation around an incident

```bash
udm correlate 01940000-0000-7000-8000-000000000005 --window 30s --output pretty
```

### Compliance audit

```bash
udm audit iso-ts-15066 --from 2026-01-01T00:00:00Z --to 2026-04-01T00:00:00Z --output pretty
```

Supported standards at v0.0.3: `iso-ts-15066`, `iso-13482`, `ansi-ria-r15.06`, `iso-3691-4`.

### Fleet metrics

```bash
udm aggregate --field power/battery/soc_pct --by source_type --agg avg
```

## Filter expression syntax

`--filter` (repeatable; multiple filters AND together) accepts:

| Form | Example |
|---|---|
| `field=value` | `event_type=telemetry_periodic` |
| `field!=value` | `source_type!=simulation` |
| `field in [a,b,c]` | `source_type in [amr,agv]` |
| `field contains text` | `event_type contains violation` |
| `field exists` | `safety exists` |

Values are parsed as JSON first; quoted strings work for literals with
special characters.

## License

Apache-2.0.
