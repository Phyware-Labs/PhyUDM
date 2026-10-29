# Schema Version & Compatibility Contract

> **Status:** Normative for `v0.0.x`. The full contract becomes binding at
> `v0.1.0` (the first stable OSS release). Until then, breaking changes MAY
> be made within `v0.0.x` provided each change ships with a `CHANGELOG.md`
> entry and a migration note.

This document is the authoritative versioning contract for the
**Unified Data Model (UDM)**. It covers:

- Semantic Versioning (SemVer) rules for schema changes
- The `udm_version` envelope field
- Canonical `$id` / `$schema` URL pattern
- Compatibility matrix across versions
- Deprecation policy and removal windows

---

## 1. Semantic Versioning

UDM follows [SemVer 2.0.0](https://semver.org/spec/v2.0.0.html) for the
**schema artifact** (`schemas/v<MAJOR>.<MINOR>.<PATCH>/`).

| Bump kind | Examples                                                                   | Compat |
|-----------|----------------------------------------------------------------------------|--------|
| **PATCH** | Tightening a field description; clarifying a `pattern`; adding a new value to a *non-exhaustive* enum that already lists `custom`; fixing a typo. | Backwards-compatible: every producer and every consumer continues to work without code changes. |
| **MINOR** | Adding a new optional field; adding a new domain; adding a new event-type or source-type enum variant; relaxing a constraint (e.g. raising a `maximum`). | Backwards-compatible: old producers still validate; old consumers ignore new fields but may not surface them. |
| **MAJOR** | Renaming a field; removing a field; tightening a constraint (e.g. making an optional field required, lowering a `maximum`); changing a field type; removing an enum variant that is not `custom`. | Breaking: producers and consumers MUST be re-tested. |

### Pre-`v1.0.0` carve-out

For `v0.x.y`, MINOR bumps MAY be breaking (SemVer §4). UDM uses this carve-out
sparingly: every breaking pre-`v1.0.0` change requires a `CHANGELOG.md`
entry, a migration note, and at least one release candidate (`v0.x.y-rc.N`)
before the final tag.

---

## 2. The `udm_version` Envelope Field

Every UDM event MUST include `udm_version` as a top-level envelope field:

```json
{
  "udm_version": "0.0.3",
  "event_id": "01940000-0000-7000-8000-000000000001",
  "event_type": "telemetry_periodic",
  "source_id": "amr-001",
  "source_type": "amr",
  "captured_at": "2026-06-07T19:00:00Z"
}
```

- The value MUST be a full SemVer string of the form `MAJOR.MINOR.PATCH`
  (the canonical schema enforces this with the pattern
  `^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$`).
- Producers SHOULD emit the highest schema version they are capable of
  generating.
- Consumers MUST be able to select a validator for the declared
  `udm_version`. A consumer that does not implement the declared version
  SHOULD reject the event (do not silently downgrade).

---

## 3. Canonical `$id` and `$schema` URL Pattern

Every schema artifact carries a stable `$id` of the form:

```
https://schemas.phyudm.org/v<MAJOR>.<MINOR>.<PATCH>/<path>.schema.json
```

| Schema kind     | `$id`                                                                                    |
|-----------------|------------------------------------------------------------------------------------------|
| Envelope        | `https://schemas.phyudm.org/v0.0.3/envelope.schema.json`                                 |
| Event           | `https://schemas.phyudm.org/v0.0.3/event.schema.json`                                    |
| Object Ref      | `https://schemas.phyudm.org/v0.0.3/object_ref.schema.json`                               |
| Domain          | `https://schemas.phyudm.org/v0.0.3/domains/<NN>-<slug>.schema.json`                      |

Every schema uses
[`$schema: "https://json-schema.org/draft/2020-12/schema"`](https://json-schema.org/specification.html).

**Host stability:** `schemas.phyudm.org` is the canonical host pre-`v0.1.0`.
The host MAY change before the OSS launch tag is published; once `v0.1.0`
ships, the host becomes part of the contract and any change is a MAJOR
bump.

**Resolution:** UDM schemas use *relative* `$ref` paths within a version
directory (e.g. `event.schema.json` references `envelope.schema.json` by
relative path). A conformant validator MUST be able to resolve refs from
either the canonical `$id` URL or the relative path. The reference
implementation in `tools/validate.py` demonstrates both styles.

---

## 4. Compatibility Matrix

| Producer ↓ / Consumer → | `0.0.3` | `0.1.x` (planned) | `0.2.x` (planned) | `1.0.x` (planned) |
|-------------------------|---------|-------------------|-------------------|-------------------|
| `0.0.3`                 | ✅       | ✅†               | ⚠️ partial        | ⚠️ partial        |
| `0.1.x`                 | ❌       | ✅                | ✅†               | ⚠️ partial        |
| `0.2.x`                 | ❌       | ❌ (drop legacy)  | ✅                | ✅†               |
| `1.0.x`                 | ❌       | ❌                | ❌                | ✅                |

Legend: ✅ = full compatibility; ✅† = backwards-compatible (consumer ≥
producer); ⚠️ partial = consumer can validate envelope + a subset of
domains; ❌ = incompatible (consumer rejects).

The matrix is updated with every MINOR or MAJOR release.

---

## 5. Deprecation Policy

A field, enum value, or domain may be **deprecated** in any release. The
contract:

1. **Announcement.** The deprecation MUST be noted in `CHANGELOG.md` and in
   the schema `description` for the deprecated item, prefixed with
   `DEPRECATED in vX.Y.Z:`.
2. **Deprecation window.** Deprecated items remain valid for **at least
   one MINOR release** (pre-`v1.0.0`) or **at least one MAJOR release**
   (post-`v1.0.0`). The minimum window is calendar-driven as well: a
   deprecated item MUST remain valid for at least **180 days** from the
   release that introduced the deprecation.
3. **Removal.** Removal of a deprecated item requires a MAJOR bump
   (post-`v1.0.0`) or a clearly-marked MINOR bump (pre-`v1.0.0`) and a
   migration note in `CHANGELOG.md`.
4. **Aliases.** When a field is renamed, the old name SHOULD be accepted
   for the duration of the deprecation window via a JSON Schema alias
   (e.g. an `oneOf` accepting either spelling). Producers SHOULD switch
   to the new name immediately; consumers MUST handle both.

---

## 6. Stability tiers

| Tier        | Definition                                                                                 |
|-------------|--------------------------------------------------------------------------------------------|
| **Stable**  | Field/domain has been in the spec ≥ 1 MAJOR release; covered by conformance fixtures; safe to depend on. |
| **Beta**    | New field/domain marked `description: "Beta in vX.Y.Z."`. Behavior is intended to be stable but may change with one MINOR release of notice. |
| **Experimental** | Field/domain marked `description: "Experimental in vX.Y.Z."`. May change or be removed in any release. Use behind a feature flag in producers. |

Pre-`v0.1.0` everything is implicitly Experimental.

---

## 7. Where to find versioned artifacts

- Schemas: `schemas/v<version>/`
- Conformance fixtures: `conformance/`
- Spec narrative: `spec/` (single version only — historical narrative lives
  in `CHANGELOG.md`).

---

## 8. Quick reference

```text
PATCH   →  clarifying-only changes; safe for everyone
MINOR   →  additive changes; producers safe to upgrade; consumers should
           plan to upgrade within one release cycle
MAJOR   →  breaking changes; producers + consumers re-test together
DEPREC. →  ≥ 1 minor release + ≥ 180 days before removal
```

