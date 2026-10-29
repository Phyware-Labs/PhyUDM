# Contributing to UDM

Thanks for your interest in the Unified Data Model (UDM) specification.
UDM is an open, vendor-neutral schema for representing telemetry and
events from autonomous systems.

This project is in early-stage extraction from a private monorepo and
is still stabilizing toward its `v0.1.0` OSS launch. The areas where
contributions are most welcome today are:

- Reviewing the spec for vendor neutrality and clarity
- Proposing new event types, source types, or domain fields
- Filing example payloads from real-world autonomous systems
- Improving documentation, diagrams, and conformance test ideas

## How to contribute

1. **Open an issue first** for any non-trivial change (new domain,
   breaking change to an event type, schema URL change, etc.) so the
   discussion can happen before code is written.
2. **Small fixes** (typos, broken links, clarifications) are welcome as
   direct pull requests against `main`.
3. **Schema changes** must include a CHANGELOG entry and an explanation
   of backward-compatibility impact.

## Pull request checklist

- [ ] The change is described in `CHANGELOG.md` (under an `Unreleased` heading).
- [ ] All `$schema` / `$id` URLs use the canonical UDM host.
- [ ] Spec text remains vendor-neutral (no proprietary product names).
- [ ] Examples validate against the JSON Schema artifacts (once published).

## Developer Certificate of Origin

By contributing to this repository you agree to the
[Developer Certificate of Origin](https://developercertificate.org/).
Sign your commits with `git commit -s`.

## Code of Conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md).
By participating, you agree to abide by its terms.

## License

By contributing to this repository, you agree that your contributions
will be licensed under the [Apache License 2.0](LICENSE).
