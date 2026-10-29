# Unified Data Model (UDM) Specification

**Version:** 0.0.3  
**Status:** Draft  
**Date:** January 12, 2026

---

The Unified Data Model (UDM) specification is organized into the
following pages. The full monolithic specification is preserved at
[`udm-spec.md`](./udm-spec.md) for backward compatibility and easy
diffing across versions.

## Front matter

- [Overview](./overview.md) — what UDM is and the problem it solves.
- [Design Principles](./design-principles.md) — the guiding tenets.
- [Object References](./object-references.md) — first-class entity refs.
- [Schema Version & Compatibility](./versioning.md) — versioning
  contract and forward/backward compatibility guarantees.

## Core schema

- [Core Envelope](./envelope.md) — required fields on every event.
- [Event Types](./event-types.md) — canonical event-type taxonomy.
- [Source Types](./source-types.md) — canonical data-source enumeration.

## Domain schemas (23)

> The canonical schema artifacts (`schemas/v0.0.3/domains/*`) cover the
> 23 structured domains below. Free-form vendor or experimental data
> belongs in the event-level `extensions` field (see
> [`spec/event-example.md`](./event-example.md)); a dedicated "Extensions
> Domain" is *not* part of `v0.0.3` even though the narrative page below
> describes the design intent for a future structured form.

- [1. Identity Domain](./domains/01-identity.md)
- [2. Location Domain](./domains/02-location.md)
- [3. Motion Domain](./domains/03-motion.md)
- [4. Power Domain](./domains/04-power.md)
- [5. Operational State Domain](./domains/05-operational-state.md)
- [6. Navigation Domain](./domains/06-navigation.md)
- [7. Perception Domain](./domains/07-perception.md)
- [8. Safety Domain](./domains/08-safety.md)
- [9. Actuators Domain](./domains/09-actuators.md)
- [10. Communication Domain](./domains/10-communication.md)
- [11. Compute Domain](./domains/11-compute.md)
- [12. AI/Reasoning Domain](./domains/12-ai-reasoning.md)
- [13. Maintenance Domain](./domains/13-maintenance.md)
- [14. Context Domain](./domains/14-context.md)
- [15. Payload/Cargo Domain](./domains/15-payload-cargo.md)
- [16. Manipulation Domain](./domains/16-manipulation.md)
- [17. Human-Robot Interaction (HRI) Domain](./domains/17-human-robot-interaction.md)
- [18. Multi-Agent Coordination Domain](./domains/18-multi-agent-coordination.md)
- [19. Simulation/Digital Twin Domain](./domains/19-simulation-digital-twin.md)
- [20. Thermal Management Domain](./domains/20-thermal-management.md)
- [21. Audio Domain](./domains/21-audio.md)
- [22. Environment Interaction Domain](./domains/22-environment-interaction.md)
- [23. Compliance/Certification Domain](./domains/23-compliance-certification.md)
- [24. Extensions Domain](./domains/24-extensions.md) — *narrative only in
  `v0.0.3`; no JSON Schema artifact. Use the envelope-level `extensions`
  field. See [versioning.md](./versioning.md) for the upgrade path.*

## Reference & operational guidance

- [Complete UDM Event Example](./event-example.md)
- [OpenTelemetry Compatibility Layer](./otel.md)
- [Provenance Metadata](./provenance.md)
- [Schema Evolution Guidelines](./evolution.md)
- [Data Quality & Validation](./data-quality.md)
- [Streaming & Batching](./streaming.md)
- [Implementation Notes](./implementation-notes.md)

## Appendices

- [Appendix A — Source Type to Domain Relevance Matrix](./appendix-a.md)
- [Appendix B — Vendor Extension Registry](./appendix-b.md)
- [Appendix C — Units Reference](./appendix-c.md)

## Monolithic copy

- [`udm-spec.md`](./udm-spec.md) — the full specification in a single
  file, kept in sync with the split pages for backward compatibility.

## Specification changelog

- [Changelog](./changelog.md) — version history of the specification text.
