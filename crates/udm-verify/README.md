# udm-verify

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](../../LICENSE)

Open verifier for [PhyUDM](../../README.md) provenance. It checks signed
robot and drone telemetry, and the evidence packages a receiver seals, with
no dependency on, or trust in, the service that collected them.

- **Event signatures:** the producer's ES256 (or development HMAC)
  signature over the event's canonical signing payload, signed singly or as
  part of an RFC 9162 Merkle batch
  ([spec](../../spec/provenance.md)).
- **Ordering and replay:** each source's `(boot_counter, sequence_num)`
  stream classified as in order, gap, restart, late, duplicate, reset or
  regression.
- **Evidence packages:** a receiver's signed manifest, its Merkle root and
  hash chain, and every event kept as received re-checked independently of
  the receiver's own verdict
  ([spec](../../spec/evidence-package.md)).

The crate does no I/O of its own, so it can be compiled to WebAssembly and
embedded in other languages.

## Command line

The [`udm` CLI](../udm-cli/) wraps it:

```bash
udm verify incident.evidence.ndjson --trust-key phycloud-evidence.pem
udm verify events.ndjson --key amr-001.pem --summary
```

## Library

```rust
use udm_verify::package::{read_package, verify_package, PackageOptions};
use udm_verify::{Es256Key, Verdict};

let package = read_package(std::io::BufReader::new(std::fs::File::open("incident.evidence.ndjson")?))?;
let options = PackageOptions {
    trusted_manifest_keys: vec![Es256Key::from_pem(&std::fs::read_to_string("phycloud-evidence.pem")?)?],
    ..PackageOptions::default()
};
let report = verify_package(&package, &options);
assert_eq!(report.verdict, Verdict::Verified, "{:#?}", report.problems);
```

Single events:

```rust
use udm_verify::{Es256Key, Keyring, SignatureStatus, Verifier};

let mut keyring = Keyring::new();
keyring.add_es256(Es256Key::from_pem(&device_pem)?);
let check = Verifier::new(&keyring).check_event(&event);
assert_eq!(check.status, SignatureStatus::Verified);
```

## Trust

A package that verifies against a key it carries proves only that it is
consistent. Pin the receiver's key (`--trust-key`, or
`PackageOptions::trusted_manifest_keys`) from an independent source to get
a `verified` verdict; otherwise the verdict is `intact_unpinned`.

## Conformance

The tests run against the shared vectors in
[`conformance/vectors/`](../../conformance/vectors/):
`provenance-signing.json`, `merkle-batch.json`, and
`evidence-package.json`, which this crate generates
(`UPDATE_VECTORS=1 cargo test -p udm-verify --test package`).

## License

Apache-2.0.
