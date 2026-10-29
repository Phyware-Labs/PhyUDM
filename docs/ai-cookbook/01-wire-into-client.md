# 1. Wire UDM-MCP into Claude Desktop / Cursor / Copilot CLI

## Scenario

You've installed `udm-mcp` and want to start querying UDM telemetry
from an LLM agent — no commercial backend, no fleet manager, no
ROS-bag-to-UDM converter. Just the binary and the seeded fixture
shipped with this article.

## Setup

```bash
# 1. Install the binary (from a local checkout for now):
cargo install --path crates/udm-cli   # for `udm` (validation + ad-hoc analysis)
cargo install --path crates/udm-mcp   # for `udm-mcp` (the MCP server)

# 2. Pick the dataset:
DATA=$(pwd)/docs/ai-cookbook/datasets/wire-into-client.ndjson
```

Then wire the server into your MCP-capable client. Pick one:

### Claude Desktop

Edit `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "udm": {
      "command": "udm-mcp",
      "args": ["--store", "memory:///ABS/PATH/TO/wire-into-client.ndjson"]
    }
  }
}
```

Restart Claude Desktop. The `udm` server should now appear in the
🔌 connector menu with 10 tools.

### Cursor

Edit `~/.cursor/mcp.json` (or workspace `.cursor/mcp.json`):

```json
{
  "mcpServers": {
    "udm": {
      "command": "udm-mcp",
      "env": { "UDM_STORE": "memory:///ABS/PATH/TO/wire-into-client.ndjson" }
    }
  }
}
```

### Copilot CLI

```bash
copilot mcp add udm \
  --command udm-mcp \
  --env UDM_STORE=memory:///ABS/PATH/TO/wire-into-client.ndjson
```

## Walk-through

In your client, send this prompt:

> _"Summarize the last 10 seconds of activity for `amr-001`."_

The agent calls **`timeline`**:

```jsonc
// → tool: timeline
{
  "source_id": "amr-001",
  "time_range": {
    "from": "2026-06-07T19:00:00Z",
    "to": "2026-06-07T19:00:10Z"
  }
}

// ← result (truncated)
{
  "source_id": "amr-001",
  "event_count": 3,
  "events": [
    { "event_id": "01940000-...-000000000001", "event_type": "telemetry_periodic",
      "captured_at": "2026-06-07T19:00:00Z",
      "power": { "battery": { "soc_pct": 78.4 } } },
    { "event_id": "01940000-...-000000000002", "event_type": "telemetry_periodic",
      "captured_at": "2026-06-07T19:00:02Z",
      "power": { "battery": { "soc_pct": 78.2 } } },
    { "event_id": "01940000-...-000000000003", "event_type": "safety_violation",
      "captured_at": "2026-06-07T19:00:05Z",
      "safety": { "safety_state": "warning", "violation_count": 1 } }
  ]
}
```

## Final agent output

> In the 10-second window starting at `2026-06-07T19:00:00Z`, `amr-001`
> emitted two periodic telemetry events (battery state-of-charge fell
> from 78.4 % to 78.2 %, no other anomalies) and one safety event at
> `T+5s`: a `safety_violation` that left the safety state at
> `warning` (`violation_count = 1`). The robot did not enter a
> protective stop; supervision is recommended.

## Try it yourself

Paste verbatim:

> _"Use the udm server to find every emergency_stop event in the last
> minute and show me which robot raised each one."_

The agent should call `query_events` with
`filters: ["event_type=emergency_stop"]` and report `agv-002` at
`19:00:07Z`.

## Going to production with PhyCloud

When you're ready to hook the agent up to a live PhyCloud-backed
fleet, swap the `--store` URL:

```jsonc
"args": ["--store", "phycloud://api.phycloud.example.com?token=YOUR_KEY"]
```

> **Note:** the `phycloud://` adapter implements the read path (query, get,
> timeline, aggregate). Add `session_id=<mission>` to scope it to one mission.
