# UDM Conformance Suite

This directory holds the language-agnostic conformance suite for the
**Unified Data Model (UDM)** specification. Any third-party UDM validator —
not just the `tools/validate.py` reference implementation shipped in this
repository — should be able to consume these fixtures and produce
deterministic pass/fail outcomes.

## Layout

```
conformance/
├── valid/      JSON payloads that MUST validate against the canonical schema
├── invalid/    JSON payloads that MUST fail validation
├── edge/       Boundary-case payloads that MUST validate (sub-second
│               timestamps, max numeric values, empty domain slots, ...)
└── legacy/     Pre-Phase-1 PhyCloud fixtures captured for drift tracking.
                These are NOT canonical and may fail validation; see each
                fixture's `*.meta.yaml` for documented drift.
```

Every `<fixture>.json` is paired with a `<fixture>.meta.yaml` describing:

| Key                          | Purpose                                                          |
|------------------------------|------------------------------------------------------------------|
| `purpose`                    | One-line description of the scenario the fixture tests.          |
| `expected_result`            | `valid`, `invalid`, or `drift_documented` (for `legacy/`).       |
| `schema_path`                | Path (relative to `schemas/v<ver>/`) of the schema to use.       |
| `target_field_path`          | Field the fixture exercises (most useful for `invalid/`).        |
| `expected_failure_keyword`   | (`invalid/` only) JSON Schema keyword expected to trip first.    |
| `domain_schema`              | (per-domain `valid/` only) The domain-specific schema referenced.|
| `notes`                      | Free-form notes.                                                 |
| `drift` (legacy only)        | List of documented drift entries, each with                      |
|                              | `field`, `actual`, `expected`, and `resolution`.                 |

## Running against a third-party validator

A conformant validator MUST:

1. Load every `*.schema.json` under `schemas/v<ver>/` and resolve `$ref`s
   relative to that directory. The reference implementation registers
   schemas by `$id`, by relative path, and by basename — that covers the
   reference styles used in the canonical schemas.
2. Iterate `valid/` and `edge/` payloads and validate each against
   `schemas/v<ver>/event.schema.json`. The suite passes if **every** payload
   validates without error.
3. Iterate `invalid/` payloads and validate each. The suite passes if
   **every** payload produces at least one validation error. Optionally
   check the reported `keyword` matches the fixture's
   `expected_failure_keyword`.
4. The `legacy/` partition is informational. Validators MAY report whether
   each fixture validates, but MUST NOT gate suite pass/fail on the result.

### Reference command

```bash
make install                      # installs jsonschema + referencing
make validate                     # runs the suite against v0.0.3 schemas
```

### Wiring your own validator

Most JSON Schema 2020-12 implementations support file-based registries.
Example (Node.js, [ajv](https://ajv.js.org)):

```js
const Ajv = require("ajv/dist/2020").default;
const addFormats = require("ajv-formats");
const fs = require("fs"), path = require("path");

const ajv = new Ajv({ allErrors: true, strict: false });
addFormats(ajv);
for (const f of walk("schemas/v0.0.3")) {
  ajv.addSchema(JSON.parse(fs.readFileSync(f, "utf8")));
}
const validateEvent = ajv.getSchema(
  "https://schemas.phyudm.org/v0.0.3/event.schema.json",
);
```

Example (Go, [santhosh-tekuri/jsonschema](https://github.com/santhosh-tekuri/jsonschema)):

```go
import "github.com/santhosh-tekuri/jsonschema/v5"

c := jsonschema.NewCompiler()
// Load every schema under schemas/v0.0.3 ahead of time, then:
sch, _ := c.Compile("schemas/v0.0.3/event.schema.json")
err := sch.Validate(payload)
```

## Adding fixtures

Fixtures are **generated** by `tools/build_conformance.py`. Hand-editing the
JSON / meta.yaml files in this directory will be overwritten by the next
`make build-conformance`. To add a fixture:

1. Add a template to the appropriate generator in
   `tools/build_conformance.py` (e.g. add to the `bad_enum_cases` list for a
   new envelope enum-violation case).
2. Run `make build` to regenerate both schemas and conformance.
3. Run `make validate` to confirm the new fixture behaves as advertised.
4. Update `CHANGELOG.md` under `[Unreleased]` describing the addition.

## Versioning and stability

This suite is versioned alongside the schemas. Fixtures live in the same
repository as the schemas they target; for each future schema version
(`schemas/v<ver>/`) the matching fixtures are emitted into this directory.
A fixture removed in a future version SHOULD be retained in the
`legacy/` partition with a `drift` entry documenting why it no longer
applies.
