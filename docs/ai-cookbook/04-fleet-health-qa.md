# 4. Fleet health Q&A — live telemetry as a conversation

## Scenario

You're sitting next to a fleet operator at a warehouse-east logistics
hub. They want to ask plain-English questions about the 8 robots
they're responsible for — no SQL, no Grafana dashboard, no swivel to
a vendor portal. The MCP server's `query_events` + `aggregate` +
`timeline` tools, pointed at a live telemetry store, are enough.

The seeded dataset for this article is 30 minutes of recent telemetry
across 8 sources (3 AMRs, 2 AGVs, 2 cobots, 1 drone) plus a couple of
injected events (one speed-limit violation, one mode change) for the
agent to find.

## Setup

```bash
DATA=$(pwd)/docs/ai-cookbook/datasets/fleet-health-warehouse-east.ndjson
udm-mcp --store memory:///${DATA}
```

## Walk-through

### Question 1 — "Which AMRs are low on battery right now?"

The agent calls **`aggregate`** to fetch the latest battery SoC by source:

```jsonc
// → tool: aggregate
{
  "agg_fn": "min",
  "field": "power/battery/soc_pct",
  "group_by": ["source_id"],
  "filters": ["source_type=amr"]
}

// ← result (truncated)
{
  "agg_fn": "min",
  "buckets": [
    { "key": { "source_id": "amr-001" }, "value": 76.8, "event_count": 30 },
    { "key": { "source_id": "amr-002" }, "value": 20.5, "event_count": 30 },
    { "key": { "source_id": "amr-003" }, "value": 89.9, "event_count": 30 }
  ]
}
```

The agent flags `amr-002` (≈ 20 %) for the operator.

### Question 2 — "What changed for amr-002 in the last 30 minutes?"

The agent calls **`timeline`** to pull every event for that source:

```jsonc
// → tool: timeline
{
  "source_id": "amr-002",
  "time_range": { "from": "2026-06-07T12:00:00Z", "to": "2026-06-07T12:30:00Z" }
}
```

…and finds 30 periodic-telemetry events plus one `mode_change` at
`12:27:00Z` where the robot transitioned from `executing_task` to
`charging` under operational mode `maintenance`.

### Question 3 — "Were any safety violations raised in the last 30 min?"

The agent calls **`query_events`**:

```jsonc
// → tool: query_events
{
  "filters": ["event_type=safety_violation"],
  "time_range": { "from": "2026-06-07T12:00:00Z", "to": "2026-06-07T12:30:00Z" }
}

// ← result
{
  "events": [
    { "event_id": "01940000-...000000000242", "event_type": "safety_violation",
      "captured_at": "2026-06-07T12:22:00Z", "source_id": "agv-008",
      "safety": { "violation_count": 1,
                  "violations": [{ "type": "speed_exceeded",
                                   "severity": "medium" }] } }
  ],
  "next_cursor": null,
  "total_estimate": 1
}
```

### Question 4 — "How does the fleet average power draw compare across source types?"

The agent calls **`aggregate`** one more time:

```jsonc
// → tool: aggregate
{
  "agg_fn": "avg",
  "field": "power/power_consumption_w",
  "group_by": ["source_type"]
}
```

## Final agent output

> **Warehouse-east fleet status — last 30 min (12:00 → 12:30 UTC)**
>
> - **Low battery alert.** `amr-002` is sitting around 20 % SoC (min
>   observed: 20.5 %); it has already transitioned to `mode=maintenance`
>   / `state=charging` at `12:27Z` — no operator action needed.
> - **Safety event.** One `safety_violation` was raised at `12:22Z`
>   on `agv-008`: `speed_exceeded` (severity = `medium`). Nothing
>   escalated to a protective stop. Recommend a quick check on the
>   AGV's commanded velocity vs. the zone speed-limit configuration.
> - **Power profile.**
>   - `amr` average power consumption: ≈ 140 W
>   - `agv` average: ≈ 140 W
>   - `cobot` average: ≈ 140 W
>   - `drone` average: ≈ 140 W
>
>   (Distributions are tight because the synthetic dataset uses
>    a narrow RNG range; real telemetry would show wider spread.)
>
> Recommended action: prioritise resolving the `agv-008` speed issue;
> `amr-002` is self-managing.

## Try it yourself

Paste verbatim:

> _"Find the robot in fleet `warehouse-east` with the lowest battery
> right now and tell me whether it's already charging."_

The agent should call `aggregate` for the SoC minimum and `timeline`
for the candidate robot to check operational state.

## Going to production with PhyCloud

PhyCloud streams telemetry in real time, so the same `--store` URL
(`phycloud://...`) gets you a *live* conversation rather than a
30-minute snapshot. The tool semantics don't change; only the
freshness of the data does.
