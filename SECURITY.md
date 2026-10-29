# Security policy

UDM is a specification plus reference tooling: JSON Schemas, a conformance
suite, the `udm` CLI, the `udm-mcp` server and event-store adapters.

## Reporting a vulnerability

Please report suspected vulnerabilities privately through
[GitHub private vulnerability reporting](https://github.com/Phyware-Labs/PhyUDM/security/advisories/new),
not in a public issue.

Include what you found, how to reproduce it and the affected version or
commit. You can expect an acknowledgement within 5 working days and an
assessment within 15. Fixes land on `main` and in the next release.

## Scope

In scope: the Rust crates under `crates/`, the schema and conformance tooling
under `tools/`, and schema defects that let a validator accept data it should
reject.

Out of scope: hosted services that implement UDM (report those to their
operator), and third-party dependencies (report upstream; tell us if UDM
tooling is affected).
