# Evidence Packages

An evidence package is a self-contained, offline-verifiable record of a set
of events that a **receiver** (the service that collected them) has sealed:
for example, the events around an incident, handed to an investigator,
insurer or regulator. Anyone holding the package and the receiver's public
key can check, without contacting the receiver:

- that the package is exactly what the receiver sealed (a signed manifest
  and a Merkle root over its events);
- that no event was changed, removed or reordered while in the receiver's
  care (content hashes and the receiver's hash chain);
- for each event the receiver kept **as received**, that the producer
  signed it (spec [Provenance Metadata](./provenance.md)), independently of
  the receiver's own verdict; and
- the order of each source's events (gaps, replays and rollbacks).

The reference verifier is the `udm-verify` crate, and `udm verify <package>`
on the command line.

## File format

A package is UTF-8 [NDJSON](https://github.com/ndjson/ndjson-spec): one JSON
value per line. The first line is the header; each further line is one event
record, in the manifest's order (ascending `sequence_number`).

```text
{"type":"udm.evidence-package.v1","manifest":{…}}
{"type":"event","event_id":"…","sequence_number":101,"content_hash":"…","previous_hash":"…","event":{…},"received":{…}}
{"type":"event", …}
```

Blank lines are ignored. Readers MUST reject a file whose first line is not
a header with `type` `udm.evidence-package.v1`.

### Event records

| Field | Type | Meaning |
|---|---|---|
| `type` | string | `event` |
| `event_id` | string | The event's id |
| `sequence_number` | integer | The receiver's sequence number for the event (its position in the receiver's chain) |
| `content_hash` | string | The receiver's [content hash](#content-hash) of `event` |
| `previous_hash` | string | The `content_hash` of the event at `sequence_number − 1` in the receiver's chain |
| `event` | object | The event as the receiver stored it |
| `received` | object | OPTIONAL. The event exactly as the producer sent it |

Receivers often normalise what they store (for example writing timestamps
with `Z` instead of `+00:00`), which changes the producer's
[signing payload](./provenance.md#signing-payload). A receiver that keeps
the event as received lets anyone re-check the producer's signature; without
`received`, only the receiver's recorded verdict is available.

## Manifest

The manifest describes the sealed set and is signed by the receiver.

| Field | Type | Meaning |
|---|---|---|
| `manifest_version` | integer | `3` (this document). Version `2` is described under [Version 2](#version-2-manifests). |
| `issuer` | string | Who sealed the package, e.g. `phycloud` |
| `issued_at` | date-time | When the manifest was issued |
| `locker_id`, `name`, `status`, `scope`, `sealed_at`, `created_at` | | The sealed set's identity, the receiver's selection criteria, and its lifecycle |
| `event_count` | integer | Number of entries in `events` |
| `merkle_algorithm` | string | `rfc9162-sha256` |
| `merkle_root` | string | See [Merkle root](#merkle-root) |
| `events` | array | One entry per event, in order; see below |
| `absences` | array | Periods a source was silent (missed heartbeats), as recorded by the receiver |
| `keys` | array | The producers' public keys; see below |
| `signing_key` | object | OPTIONAL. `{key_id, public_key_pem}` of the key that signs the manifest, for convenience. It does not establish trust: see [Trust](#trust). |
| `signature` | object | `{algorithm, key_id, value}`; see [Signature](#signature) |

Each `events` entry:

| Field | Meaning |
|---|---|
| `event_id`, `sequence_number`, `content_hash` | As in the event record |
| `received_hash` | OPTIONAL. Lowercase hex SHA-256 of the [canonical JSON](./provenance.md#signing-payload) of the event as received (signature included) |
| `signature_status` | The receiver's verdict on the producer signature at ingest: `verified`, `unsigned`, `missing_signature`, `invalid_signature`, `unknown_key`, `key_source_mismatch`, `retired_key`, `revoked_key` or `unverifiable_algorithm` |
| `signer_key_id`, `key_binding`, `sequence_status`, `attestation` | The signing key, whether it is bound to hardware, the receiver's [ordering](./provenance.md#ordering-and-replay) verdict, and the device attestation current at capture |

Each `keys` entry: `key_id` (lowercase hex SHA-256 of the DER
SubjectPublicKeyInfo), `algorithm` (`es256`), `public_key_pem`, and
optionally `source_id` (the source the key belongs to), `binding`
(`hardware` or `software`), `retired_at` and `revoked_at`. Because the
manifest is signed, the receiver vouches for these keys and their
lifecycle; a verifier MAY also use keys it holds itself, and SHOULD prefer
them.

### Content hash

`content_hash` is the lowercase hex SHA-256 of the stored event's canonical
JSON in the **hash-chain form**. This is the canonical JSON of
[Provenance Metadata](./provenance.md#signing-payload) (keys sorted, compact,
non-ASCII escaped), `null` members kept, with one difference in how floating
point numbers are written:

| | Decimal notation for exponents | Exponent digits | Examples |
|---|---|---|---|
| Signing payload (Python `repr`) | −4 to 15 | at least two | `1e-05`, `1e-07`, `1e+16` |
| Hash-chain form | −5 to 15 | as few as needed | `0.00001`, `1e-7`, `1e+16` |

Both use the shortest digit string that reads back as the same number. In
the hash-chain form `previous_hash` links each event to the one before it
in the receiver's chain, so a removed or altered event breaks the link.
When the package skips sequence numbers (a manifest scoped to some sources
holds only part of the chain), the link across the skip cannot be checked
and verifiers report a discontinuity, not a failure.

### Merkle root

The root is computed as for [batch signatures](./provenance.md#batch-signatures)
(RFC 9162 §2.1). The leaves are the `events` entries' content hashes, in
order: leaf *i* is `SHA-256(0x00 ‖ bytes(content_hash_i))`, where `bytes` is
the 32 bytes the hex string encodes. No events give `SHA-256("")`.

### Signature

`signature.value` is the lowercase hex 64-byte `r ‖ s` ES256 signature over
the canonical JSON of the manifest **without** its `signature` member.
`signature.key_id` names the key. A manifest MAY be unsigned
(`signature` absent or `null`), in which case it proves nothing about who
sealed it.

## Trust

A verifier MUST NOT treat a package as authentic because it verifies
against a key the package itself carries: anyone can build a consistent
package with a key of their own. It needs the receiver's public key from
an independent source (the receiver's published key, a contract, a key
pinned on first use) and reports:

| Verdict | Meaning |
|---|---|
| `verified` | Every check passed and the manifest is signed by a key the verifier trusts. |
| `intact_unpinned` | Every check passed, but the signing key is not one the verifier trusts (or was not supplied): consistent, signer unconfirmed. |
| `intact_unsigned` | Every check passed, but the manifest is not signed. |
| `failed` | At least one check failed. |

Producer keys listed in `keys` are trusted to the extent the manifest is.

## Verification

A verifier:

1. checks the manifest signature (and whether its key is trusted);
2. checks `event_count` and recomputes `merkle_root` from `events`;
3. matches every record to its `events` entry by `event_id` (same
   `sequence_number` and `content_hash`; no record missing, unlisted or
   repeated) and checks records are in ascending `sequence_number`;
4. recomputes every record's `content_hash`, and checks `previous_hash`
   wherever consecutive records have consecutive sequence numbers;
5. for each record with `received`: checks `received_hash`, that its
   `event_id`, `source_id` and `event_type` match the stored event, and the
   producer signature (single or batch), then compares the result with the
   receiver's `signature_status`. A receiver claiming `verified` for an
   event whose signature does not verify fails the package;
6. classifies each source's re-verified events for
   [ordering and replay](./provenance.md#ordering-and-replay) and reports
   open gaps.

## Version 2 manifests

Version 2 manifests have no `keys`, `signing_key` or `received_hash`, and
use `merkle_algorithm` `sha256-sorted-pairs`: each level hashes the text
concatenation of each pair of lowercase hex hashes, smaller first, repeating
the last hash of an odd level. Verifiers accept it with a warning; version 3
manifests use the RFC 9162 tree, which also fixes the order of events.

## Conformance

[`conformance/vectors/evidence-package.json`](../conformance/vectors/evidence-package.json)
holds the receiver's key and a set of packages, valid and tampered, each
with the verdict and error kinds a verifier must report.
