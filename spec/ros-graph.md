# ROS 2 Graph Integrity Events

A signature proves who sent an event and that it was not changed afterwards.
It does not prove the values were true when the producer received them. On a
ROS 2 robot, a process with access to the robot's network can start its own
node, subscribe to a raw sensor topic and publish altered values on the topic
everyone else reads. Every downstream signature is then valid over false data.

Forging values that way requires changing the **computation graph**: a new
node, a new subscriber on the raw topic, a new or different publisher on the
watched topic. ROS 2 discovery announces those changes to every participant,
so a producer can watch the graph passively and report them as signed events.

## Event types

| `event_type` (schema) | Spec form | Meaning |
|---|---|---|
| `security_ros_graph_baseline` | `security.ros_graph_baseline` | The observer captured the graph it will compare against (when the device is armed). Carries the manifest. |
| `security_ros_graph_changed` | `security.ros_graph_changed` | The graph departed from the baseline in a way the policy watches. Carries the changes. |
| `security_ros_graph_unobservable` | `security.ros_graph_unobservable` | The observer can no longer see the graph, e.g. discovery traffic stopped reaching it. Blinding the observer is itself a signal. |

All three carry a `ros_graph` block and SHOULD be signed like any other event
(see [Provenance Metadata](./provenance.md)), so that they cannot be quietly
removed later.

## The `ros_graph` block

| Field | Type | Meaning |
|---|---|---|
| `severity` | `info` \| `low` \| `medium` \| `high` | The highest severity among `changes` (`info` for a baseline). |
| `baseline_sha256` | hex | SHA-256 of the canonical JSON (see [Provenance](./provenance.md#signing-payload)) of the baseline manifest the event compares against. |
| `policy_id` | string | Identifier of the graph policy in force. |
| `manifest` | object | The baseline (baseline events only): `{"topics": {"<topic>": {"type", "publishers": [endpoint], "subscribers": [endpoint]}}}` for every watched topic. |
| `changes` | array | What changed (changed events): see below. |
| `reason` | string | Why the graph cannot be observed (unobservable events). |
| `observed_at` | date-time | When the observer saw the graph in this state. |

An **endpoint** describes a publisher or subscriber as discovery reports it:
`node` (fully qualified name, required), `gid` (endpoint GID, lowercase hex),
`topic_type`, `qos`, and `address` (the participant's network address, when
the observer can see it).

Each **change** has a `kind`, a `severity`, and as relevant the `topic`, the
`node`, and the endpoint `before` and `after`:

| `kind` | Meaning | Recommended severity |
|---|---|---|
| `node_added`, `node_removed` | A node appeared or disappeared | `medium` (`info` if allow-listed churn) |
| `publisher_added` | A new publisher on a watched topic | `high` |
| `publisher_removed` | A watched topic lost a publisher | `medium` |
| `publisher_replaced` | A watched topic's publisher was swapped for a different node or address | `high` |
| `publisher_restarted` | Same node name, new endpoint GID | `low` on its own; `high` if the address also changed |
| `subscriber_added` | A new subscriber on a watched raw topic | `high` |
| `subscriber_removed` | A watched subscriber went away | `low` |
| `participant_unknown` | A participant from an address the policy does not know | `high` |

## Example

```json
{
  "event_type": "security_ros_graph_changed",
  "ros_graph": {
    "severity": "high",
    "baseline_sha256": "9c9c…",
    "changes": [
      {"kind": "node_added", "severity": "medium", "node": "/speed_governor"},
      {"kind": "publisher_replaced", "severity": "high", "topic": "/odom",
       "before": {"node": "/odom_relay", "gid": "0101…"},
       "after": {"node": "/speed_governor", "gid": "0202…", "address": "192.168.4.58"}},
      {"kind": "subscriber_added", "severity": "high", "topic": "/base/odom_raw",
       "after": {"node": "/speed_governor", "gid": "0202…"}}
    ],
    "observed_at": "2026-06-07T19:21:57Z"
  }
}
```

## Limitations

- **Injection into a running process** (e.g. a preloaded library or a
  debugger) changes nothing in the graph. Only a value-level check, such as
  comparing a base's own measurements with what the main computer reports,
  can catch it.
- **Patching an allowed node and restarting it** appears as a
  `publisher_restarted` change: flagged, but low-confidence on its own.
- **DDS-Security** hides discovery from participants without permissions,
  and middleware with a different discovery mechanism needs its own observer.
- The events show that the graph changed. They do not prove what a node
  intended, or that any value is true.
