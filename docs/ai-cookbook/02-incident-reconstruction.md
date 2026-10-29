# 2. LLM-driven incident reconstruction

## Scenario

An operator pages you at 02:00:

> _"`amr-014` hit an emergency stop somewhere around `14:22` UTC. Can
> you figure out what happened?"_

The seeded dataset shipped with this article contains 14 events
around that incident, spanning normal telemetry, sensor degradation,
proximity warnings, path-blocked + rerouting, and the e-stop itself.

## Setup

```bash
DATA=$(pwd)/docs/ai-cookbook/datasets/incident-amr-014.ndjson
udm-mcp --store memory:///${DATA}  # or wire via your MCP client config
```

## Walk-through

### Step 1 — pull the incident bundle

In your client, send:

> _"Reconstruct the incident around `2026-03-14T14:22:03Z` for source
> `amr-014`. Use a 2-minute window."_

The agent calls **`incident_reconstruction`**:

```jsonc
// → tool: incident_reconstruction
{
  "source_id": "amr-014",
  "timestamp": "2026-03-14T14:22:03Z",
  "window": "2m"
}

// ← result (truncated)
{
  "source_id": "amr-014",
  "centre": "2026-03-14T14:22:03Z",
  "window_secs": 120,
  "event_count": 8,
  "events": [
    { "event_id": "01940000-...000000000007", "event_type": "error",
      "captured_at": "2026-03-14T14:21:00Z",
      "perception": { "lidar": [{ "id": "front-1", "status": "degraded" }] } },
    { "event_id": "01940000-...000000000008", "event_type": "error",
      "captured_at": "2026-03-14T14:21:15Z",
      "perception": { "lidar": [{ "id": "front-1", "status": "error" }] } },
    { "event_id": "01940000-...000000000009", "event_type": "safety_violation",
      "captured_at": "2026-03-14T14:21:40Z",
      "safety": { "violations": [{ "type": "proximity_violation",
                                    "severity": "high",
                                    "distance_m": 0.35 }] } },
    { "event_id": "01940000-...000000000010", "event_type": "path_blocked",
      "captured_at": "2026-03-14T14:21:50Z",
      "navigation": { "path": { "state": "blocked" },
                      "obstacle_count": 1 } },
    { "event_id": "01940000-...000000000011", "event_type": "rerouting",
      "captured_at": "2026-03-14T14:21:55Z",
      "navigation": { "path": { "state": "planning" } } },
    { "event_id": "01940000-...000000000012", "event_type": "emergency_stop",
      "captured_at": "2026-03-14T14:22:03Z",
      "safety": { "e_stop": { "type": "hardware" },
                  "protective_stop_reason": "front_lidar_failure",
                  "violation_count": 2 } },
    { "event_id": "01940000-...000000000013", "event_type": "state_transition",
      "captured_at": "2026-03-14T14:22:05Z",
      "operational": { "mode": "emergency",
                       "state": "emergency_stopped",
                       "previous_state": "navigating" } },
    { "event_id": "01940000-...000000000014", "event_type": "system_shutdown",
      "captured_at": "2026-03-14T14:22:48Z",
      "operational": { "mode": "maintenance", "state": "shutting_down" } }
  ]
}
```

### Step 2 — correlate across domains

The agent calls **`correlate_events`** on the e-stop event id to
double-check whether anything in the navigation or perception domains
got missed:

```jsonc
// → tool: correlate_events
{
  "seed_event_id": "01940000-0000-7000-8000-000000000012",
  "window": "60s",
  "domains": ["perception", "navigation", "safety"]
}
```

…and finds the same chain of perception → safety → navigation events
already surfaced by `incident_reconstruction`. No additional sources
in this fixture contributed to the incident.

### Step 3 — confirm the spec semantics

Before writing up the analysis, the agent calls **`explain_field`**
to confirm what the `protective_stop_reason` value means:

```jsonc
// → tool: explain_field
{ "path": "safety/protective_stop_reason" }

// ← result
{
  "path": "safety/protective_stop_reason",
  "schema_version": "0.0.3",
  "type": "string",
  "description": null
}
```

(Free-form string; the value `"front_lidar_failure"` is a
vendor-supplied reason — not a canonical enum.)

## Final agent output

> **Incident summary — `amr-014`, `2026-03-14T14:22:03Z`**
>
> A hardware emergency stop was triggered by a front-LiDAR failure
> after a 75-second degradation chain:
>
> 1. **`T-63s` (14:21:00Z)** — Front LiDAR (`front-1`) entered
>    `degraded` status (`event_id 000000000007`).
> 2. **`T-48s` (14:21:15Z)** — Front LiDAR escalated to `error`
>    (`event_id 000000000008`). At this point the perception stack
>    was operating without forward sensing.
> 3. **`T-23s` (14:21:40Z)** — High-severity `proximity_violation` at
>    0.35 m (`event_id 000000000009`) — almost certainly a false
>    positive driven by the missing LiDAR.
> 4. **`T-13s` (14:21:50Z)** — Path-blocked detected; navigation
>    enters `blocked` state with one obstacle (`event_id 000000000010`).
> 5. **`T-8s` (14:21:55Z)** — Rerouting initiated (`event_id 000000000011`).
> 6. **`T+0s` (14:22:03Z)** — Hardware e-stop, `protective_stop_reason =
>    front_lidar_failure` (`event_id 000000000012`). Operational state
>    transitioned `navigating → emergency_stopped` 2 s later, and the
>    robot shut down at `T+45s`.
>
> **Root cause:** front-LiDAR hardware failure. The proximity violation
> at `T-23s` and the blocked path at `T-13s` were downstream symptoms,
> not independent causes.
>
> **Recommendation:** swap the front-LiDAR unit, then bring the robot
> back up. Audit the rest of the fleet for the same firmware revision.

## Try it yourself

Paste verbatim:

> _"For source `amr-014` on March 14 2026, find every perception domain
> event between `14:21:00Z` and `14:22:10Z` and tell me whether the
> LiDAR was healthy at any point in that window."_

The agent should call `query_events` with a `perception exists`
predicate and report that LiDAR `front-1` was `degraded` at
`14:21:00Z` and `error` from `14:21:15Z` onward.

## Going to production with PhyCloud

For incidents that span hours or days of telemetry across thousands
of events, swap the `--store` to `phycloud://...`. Tool semantics are
identical; the backend changes.
