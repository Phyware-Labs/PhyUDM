# UDM toolchain

Maintenance scripts for the canonical schemas and conformance suite.

| Script                  | Purpose                                                                         |
|-------------------------|---------------------------------------------------------------------------------|
| `build_schemas.py`      | Regenerate `schemas/v<ver>/*.schema.json` from the in-script Rust-derived model. |
| `build_conformance.py`  | Regenerate `conformance/{valid,invalid,edge,legacy}` fixtures + `*.meta.yaml`.   |
| `validate.py`           | Reference validator (Draft 2020-12). Runs single files or the full suite.       |
| `changelog_check.py`    | Enforce `CHANGELOG.md` updates when `schemas/` or `spec/` change.               |

Everything is wired into the top-level [`Makefile`](../Makefile):

```bash
make install            # install Python deps (jsonschema, referencing, ...)
make build              # regenerate schemas/ + conformance/
make validate           # run the full conformance suite
make changelog-check    # local mirror of the CI changelog guard
make check              # validate + changelog-check
```

## Why a Python builder?

The hand-coded definitions in `build_schemas.py` are the **first-pass**
encoding of the UDM domain surface. They are derived from the Rust SDK
(`PhyTrace/rust-sdk/src/models/`) which is the de-facto source of truth
today.

The medium-term plan (tracked as `udm-rust-sdk-consume` in the parent
epic) is to replace this generator with `schemars`-derived schemas
emitted directly from the Rust models, eliminating the manual drift
surface. Until then, regenerating schemas requires:

1. Updating the field list in `build_schemas.py` to match the Rust struct.
2. `make build` to emit the JSON.
3. `make validate` to ensure conformance fixtures still pass.
4. A `CHANGELOG.md` entry under `[Unreleased]`.

## Why the conformance fixtures are generated

Hand-authoring 100+ near-duplicate JSON files is error-prone. The
generator keeps the fixture surface deterministic and reviewable: every
fixture has a documented `purpose` and `expected_result`, and the set
can be regenerated bit-for-bit on any machine. Adding a fixture means
adding a template entry to `build_conformance.py`, not committing
another raw JSON file.

Generated JSON files are still committed to the repo so third-party
validators can consume them without running the generator.
