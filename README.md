# Unified Data Model (UDM)

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](./LICENSE)
[![Spec Version](https://img.shields.io/badge/spec-v0.0.3-orange.svg)](./spec/udm-spec.md)
[![Status: Pre-launch](https://img.shields.io/badge/status-pre--launch-yellow.svg)](#status)

> **UDM** is an open, vendor-neutral specification for representing
> telemetry and events from autonomous systems — mobile robots,
> manipulators, AGVs/AMRs, humanoids, drones, and the fleets they
> belong to. UDM gives operators, regulators, and AI agents a single
> common language for physical-world data, regardless of who built
> the robot.

- 📖 **Read the spec** → [`spec/index.md`](./spec/index.md)
- 🧰 **Single-file copy for diffing** → [`spec/udm-spec.md`](./spec/udm-spec.md)
- 📐 **JSON Schema artifacts** → [`schemas/v0.0.3/`](./schemas/v0.0.3/)
- ✅ **Conformance suite** → [`conformance/`](./conformance/)
- 🏷️ **Versioning contract** → [`spec/versioning.md`](./spec/versioning.md)
- 📝 **Change history** → [`CHANGELOG.md`](./CHANGELOG.md)
- 🤝 **Contribute** → [`CONTRIBUTING.md`](./CONTRIBUTING.md)

---

## Why UDM?

Autonomous-system telemetry today is fragmented across vendors and
platforms. UDM provides a **stable schema** for the data every
autonomous system produces — identity, location, motion, power,
safety, perception, manipulation, AI/reasoning, and 17 other
domains — so the same payload can flow through:

- compliance and safety auditing (ISO 13482, ANSI/RIA R15.06,
  ISO/TS 15066, SOC 2, GDPR, …),
- fleet management and observability backends,
- LLM agents and autonomous coding pipelines, and
- digital-twin and simulation environments,

without per-vendor adapters.

---

<a id="audiences"></a>

## Who this repo is for

### 1. Spec readers (architects, standards bodies, integrators)

Start with [`spec/index.md`](./spec/index.md). The navigation hub links
to every front-matter, core-schema, domain, reference, and appendix
page. The full single-file copy lives at [`spec/udm-spec.md`](./spec/udm-spec.md)
for offline reading and version-to-version `git diff`.

Highlights:

- [Overview](./spec/overview.md) — what UDM is and the problem it solves.
- [Core Envelope](./spec/envelope.md) — required fields on every event.
- [Event Types](./spec/event-types.md) — canonical event-type taxonomy.
- [Source Types](./spec/source-types.md) — canonical data-source enumeration.
- [Schema Version & Compatibility](./spec/versioning.md) — SemVer rules,
  `$id` URL pattern, deprecation policy, compatibility matrix.
- [23 domain schemas](./spec/index.md#domain-schemas-23) — from Identity
  through Compliance.
- [Vendor Extension Registry](./spec/appendix-b.md) — how vendors register
  reserved namespaces without polluting the standard.

### 2. Autonomous-system implementers (robotics teams, platform builders)

Want to emit UDM from a robot, simulator, or fleet manager? Pair the
[Core Envelope](./spec/envelope.md) with the relevant
[domain pages](./spec/index.md#domain-schemas-23), then validate your
output with the canonical JSON Schema (`make validate`). The
[Complete UDM Event Example](./spec/event-example.md)
shows a fully-populated event payload spanning multiple domains.

> **Phase 2 — landed.** Canonical JSON Schema (Draft 2020-12) artifacts
> live under [`schemas/v0.0.3/`](./schemas/v0.0.3/); a 129-fixture
> conformance suite (67 valid + 46 invalid + 15 edge + 1 legacy) lives
> under [`conformance/`](./conformance/); the versioning contract is at
> [`spec/versioning.md`](./spec/versioning.md). Run `make validate` to
> exercise the suite locally.

### 3. AI / agent developers (LLM tooling, autonomous codegen)

UDM is designed to be the path of least resistance for LLM agents
producing or consuming autonomous-system telemetry. The strategic
goal is for any agent to be able to:

1. **Discover** the schema (via the JSON Schema artifacts and the
   Model Context Protocol server).
2. **Emit** conforming UDM directly, without per-vendor glue.
3. **Validate** their output (via the `udm` CLI).

Available today: the [`udm` CLI](./crates/udm-cli/) (validate, explore and
analyse UDM telemetry), the [`udm-mcp` server](./crates/udm-mcp/) (Model Context
Protocol tools for LLM agents) and the [AI cookbook](./docs/ai-cookbook/).

---

## Quickstart (spec reader)

```bash
git clone https://github.com/Phyware-Labs/PhyUDM.git
cd PhyUDM
$EDITOR spec/index.md           # navigate the split tree
$EDITOR spec/udm-spec.md        # or read the full single-file copy
```

A minimal UDM event looks like:

```jsonc
{
  "udm_version": "0.0.3",
  "event_id": "01940000-0000-7000-8000-000000000001",
  "event_type": "telemetry_periodic",
  "source_id": "robot-001",
  "source_type": "amr",
  "captured_at": "2026-01-12T10:35:00.123456Z",
  "identity": {
    "source_id": "robot-001",
    "source_type": "amr"
  },
  "location": {
    "frame_id": "map",
    "local": { "x_m": 12.34, "y_m": 56.78, "z_m": 0.0 }
  }
}
```

See [`spec/envelope.md`](./spec/envelope.md) for required vs. optional
envelope fields and [`spec/event-example.md`](./spec/event-example.md)
for a fully-worked multi-domain example.

---

<a id="status"></a>

## Status

| Aspect | State |
|---|---|
| Specification text | `v0.0.3` (draft) — see [`CHANGELOG.md`](./CHANGELOG.md) |
| JSON Schema artifacts | ✅ [`schemas/v0.0.3/`](./schemas/v0.0.3/) (Phase 2) |
| Conformance suite | ✅ [`conformance/`](./conformance/) — 129 fixtures (Phase 2) |
| Versioning contract | ✅ [`spec/versioning.md`](./spec/versioning.md) (Phase 2) |
| CLI | ✅ [`crates/udm-cli/`](./crates/udm-cli/) |
| MCP server | ✅ [`crates/udm-mcp/`](./crates/udm-mcp/) |
| Event-store adapters | ✅ [`crates/udm-eventstore/`](./crates/udm-eventstore/) trait, in-memory and PhyCloud (read path) adapters |
| AI cookbook | ✅ [`docs/ai-cookbook/`](./docs/ai-cookbook/) |

---

## Governance

UDM is currently maintained under **single-maintainer stewardship** by
the original authors during the pre-`v0.1.0` extraction and stabilization
period. Report vulnerabilities as described in [`SECURITY.md`](./SECURITY.md).
A formal `GOVERNANCE.md` will land before the `v0.1.0` launch.

The goal is for UDM to become an open de facto standard for
autonomous-system telemetry. Cross-vendor governance (e.g., foundation
donation) is explicitly out of scope until after `v0.1.0` so the surface
can stabilize first.

---

## Contributing

Pull requests are welcome. Please read [`CONTRIBUTING.md`](./CONTRIBUTING.md)
before opening a non-trivial change, and sign your commits (`git commit -s`)
per the Developer Certificate of Origin.

This project adopts the [Contributor Covenant](./CODE_OF_CONDUCT.md) Code
of Conduct. By participating, you agree to abide by its terms.

---

## License

UDM is licensed under the [Apache License 2.0](./LICENSE) — spec text,
schema artifacts, and tooling. See [`NOTICE`](./NOTICE)
for attribution and extraction provenance.
