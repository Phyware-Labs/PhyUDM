# 3. Compliance audit with an LLM agent

## Scenario

A safety auditor asks:

> _"Show me every ISO/TS 15066 contact event over Q1 2026 where the
> body-region force exceeded the Annex-A limit. Group by `source_id`,
> include `event_id` for traceability."_

ISO/TS 15066 (collaborative robots, biomechanical limits) maps onto
several UDM event types — `safety_violation`, `emergency_stop`,
`task_started` (collaborative-mode entry), `task_completed`
(collaborative-mode exit). The seeded dataset for this article
contains 195 events across 3 cobots (`cobot-101`/`102`/`103`) spanning
Q1 2026, with realistic force-limit violations sprinkled in.

## Setup

```bash
DATA=$(pwd)/docs/ai-cookbook/datasets/compliance-iso-ts-15066-q1.ndjson
udm-mcp --store memory:///${DATA}
```

## Walk-through

### Step 1 — bulk compliance evidence

The agent calls **`compliance_audit`**:

```jsonc
// → tool: compliance_audit
{
  "standard": "iso-ts-15066",
  "time_range": {
    "from": "2026-01-01T00:00:00Z",
    "to":   "2026-04-01T00:00:00Z"
  }
}

// ← result (truncated)
{
  "standard": "iso-ts-15066",
  "name": "ISO/TS 15066 — Collaborative robots, biomechanical limits",
  "notes": "Inspect every collaborative-mode transition and contact event; cross-check against biomechanical force/pressure limits (Annex A).",
  "from": "2026-01-01T00:00:00Z",
  "to":   "2026-04-01T00:00:00Z",
  "matching_event_count": 195,
  "events": [
    /* every safety_violation, emergency_stop, task_started, task_completed
       event in the window — 195 in total (13 weekly cycles × 3 cobots ×
       ~5 events per cycle) */
  ]
}
```

### Step 2 — narrow to force-limit violations + group

The agent now wants only the `force_limit_exceeded` violations, grouped
by source. It calls **`query_events`** with a `Contains` predicate
over the violation `type`:

```jsonc
// → tool: query_events
{
  "filters": ["event_type=safety_violation"],
  "time_range": { "from": "2026-01-01T00:00:00Z", "to": "2026-04-01T00:00:00Z" },
  "limit": 1000
}
```

…and gets back a page of `safety_violation` events. For each one with a
non-empty `safety.violations[]`, the agent inspects the `type` field
to confirm `force_limit_exceeded`.

Then it calls **`aggregate`** for the group counts:

```jsonc
// → tool: aggregate
{
  "agg_fn": "count",
  "group_by": ["source_id"],
  "filters": ["event_type=safety_violation"],
  "time_range": { "from": "2026-01-01T00:00:00Z", "to": "2026-04-01T00:00:00Z" }
}

// ← result (illustrative — exact bucket counts depend on the seeded RNG)
{
  "agg_fn": "count",
  "buckets": [
    { "key": { "source_id": "cobot-101" }, "value": 21.0, "event_count": 21 },
    { "key": { "source_id": "cobot-102" }, "value": 24.0, "event_count": 24 },
    { "key": { "source_id": "cobot-103" }, "value": 22.0, "event_count": 22 }
  ]
}
```

### Step 3 — pull provenance for the auditor's report

For each violation event, the agent calls **`get_event`** with
`include_provenance: true` so the auditor's spreadsheet has a stable
citation per row:

```jsonc
// → tool: get_event
{ "event_id": "01940000-0000-7000-8000-000000000002",
  "include_provenance": true }
```

## Final agent output

> **ISO/TS 15066 audit — Q1 2026 (2026-01-01 → 2026-04-01)**
>
> Across 3 cobots (`cobot-101`, `cobot-102`, `cobot-103`), the
> `compliance_audit` tool returned 195 in-scope events
> (collaborative-mode entry/exit + safety_violation + emergency_stop).
> Of those, **67 were `safety_violation`s carrying `force_limit_exceeded`
> entries** with `force_n > 100 N` (Annex-A hand-back threshold).
> Per-source counts:
>
> | Source | Platform | safety_violation events | Sample event id |
> |---|---|---|---|
> | cobot-101 | UR10e | 21 | `01940000-0000-7000-8000-000000000002` |
> | cobot-102 | Franka-Panda | 24 | `01940000-0000-7000-8000-000000000067` |
> | cobot-103 | Kuka-LBR-iiwa | 22 | `01940000-0000-7000-8000-000000000132` |
>
> All three cobots are flagged as ISO-13849 PL-d in
> `compliance.functional_safety.performance_level`, and every violation
> event carries `force_n > 100` in `safety.violations[0].force_n`.
> **Recommended action:** re-calibrate the impedance-control threshold
> on `cobot-102` (highest violation count) and audit the programmed
> contact forces against task-specific Annex-A limits.

## Try it yourself

Paste verbatim:

> _"Pull every ISO/TS 15066 violation in Q1 2026 for cobot-103,
> aggregate by week, and tell me which week had the highest force
> count."_

The agent should call `compliance_audit` scoped to `cobot-103`, then
either `aggregate` over week-bucketed timestamps, or fetch the events
and bucket them client-side.

## Going to production with PhyCloud

PhyCloud tracks per-event signing keys + chain-of-custody hashes in
the `provenance` block. Through the PhyCloud adapter, every
audited event includes a cryptographically-verifiable trail that an
external auditor can re-verify against PhyCloud's HMAC service
without trusting this MCP server — same prompts, stronger guarantees.
