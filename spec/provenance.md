# Provenance Metadata

Events MAY carry a `provenance` block: integrity metadata written by the
**producer** (the agent on or near the device) and covered by its signature.
It lets any receiver, or anyone later holding the event, check who produced
it, that it has not been changed, where it sits in the producer's sequence,
and what the device's attested state was.

```json
{
  "provenance": {
    "signer": "tpm",
    "algorithm": "es256",
    "key_id": "5f1c…(64 hex)",
    "signed_at": "2026-06-07T19:00:00.250000+00:00",
    "sequence_num": 42,
    "boot_epoch": "0123456789abcdef0123456789abcdef",
    "boot_counter": 7,
    "agent_id": "agent-amr-001",
    "device_attestation": {
      "mode": "tpm",
      "quote_id": "q-2026-06-07-001",
      "quote_status": "verified"
    },
    "signature": "ab12…(128 hex)"
  }
}
```

All fields are OPTIONAL. An event without `provenance` is still a valid UDM
event: provenance never becomes a requirement of the core schema.

## Fields

| Field | Type | Meaning |
|---|---|---|
| `signer` | string | Class of signer: `hmac` (shared secret) or `tpm` (non-exportable hardware key). Other values MAY be used for other key stores. |
| `algorithm` | string | Signature algorithm; see [Algorithms](#algorithms). |
| `key_id` | string | Identifier of the signing key. For `es256`, the lowercase hex SHA-256 of the key's DER `SubjectPublicKeyInfo`. |
| `signature` | string | Lowercase hex signature over the [signing payload](#signing-payload). Absent when `batch` is present. |
| `signed_at` | date-time | When the producer signed the event. |
| `sequence_num` | integer ≥ 1 | Per-source sequence number; see [Ordering](#ordering-and-replay). |
| `boot_epoch` | string | Random identifier generated each time the producer starts. |
| `boot_counter` | integer ≥ 0 | Hardware-backed count of producer starts; never decreases. |
| `agent_id` | string | Identifier of the producing agent instance. |
| `device_attestation` | object | The device's attestation state; see [Device attestation](#device-attestation). |
| `batch` | object | A Merkle batch signature covering this event; see [Batch signatures](#batch-signatures). |
| `signed_fields` | array | **Deprecated.** Signatures cover the whole event; receivers MUST ignore this field. |

Producers MAY add further fields. Every field in the block, known or not, is
covered by the signature.

## Signing payload

The producer signs the event exactly as sent, with two things removed:

1. every object member whose value is `null`, at any depth (array elements
   that are `null` are kept); and
2. `provenance.signature`. If that leaves `provenance` empty, the
   `provenance` member is removed too.

The result is rendered as canonical JSON: object keys sorted by code point,
no insignificant whitespace (`,` and `:` separators), and every non-ASCII
character escaped as `\uXXXX` (UTF-16 surrogate pairs above U+FFFF). This is
the output of Python's
`json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)`.
The signature is computed over the UTF-8 bytes of that string.

Consequences:

- The whole event is covered, including every domain payload, the
  extensions and the rest of the `provenance` block (algorithm, key id,
  sequence and boot fields, attestation). There is no field list.
- The producer builds the complete `provenance` block first, signs, then
  adds `signature`.
- A receiver that re-serialises an event (for example, normalising
  timestamps) changes the payload. A receiver that needs to re-check the
  signature later MUST keep the event as received.

[`conformance/vectors/provenance-signing.json`](../conformance/vectors/provenance-signing.json)
holds events, their signing payloads and reference HMACs. Implementations
MUST reproduce every payload byte for byte.

## Algorithms

| `algorithm` | Key | `signature` encoding | Use |
|---|---|---|---|
| `es256` | ECDSA P-256, SHA-256 | Lowercase hex of the 64-byte `r ‖ s` (as in JWS, RFC 7518 §3.4) | Production. The private key SHOULD be non-exportable hardware (e.g. a TPM). |
| `hmac-sha256` | Shared secret | Lowercase hex HMAC-SHA256 | Development and testing only: anyone who can verify can also forge. |

Verifiers SHOULD accept both `s` and `n − s` for `es256`. Other algorithm
values are reserved for future versions.

## Ordering and replay

- **`sequence_num`** starts at 1 for a source and never decreases while the
  producer's state is intact. A producer that persists its counter MUST NOT
  reuse a number after a crash (skipping numbers is allowed).
- **`boot_epoch`** changes on every producer start, so a receiver can tell a
  restart from a replay.
- **`boot_counter`**, when present, comes from a monotonic hardware counter
  incremented once per producer start (for example a TPM NV counter).
  Receivers SHOULD order a source's events by `(boot_counter, sequence_num)`:
  rolling back a producer's stored sequence state then cannot make new
  events look older.

A receiver tracking the highest `(boot_counter, sequence_num)` per source can
classify each signed event as in order, late (fills a gap), a duplicate
(replay), or a regression, and report gaps. These fields only count once
the signature over them has been verified.

## Device attestation

`device_attestation` records the device's boot state when the event was
captured:

| Field | Meaning |
|---|---|
| `mode` | `tpm`: attested by a TPM quote. `software`: no hardware root of trust. |
| `quote_id` | Identifier of the device's latest quote, as issued by the receiver that verified it. |
| `quote_status` | That receiver's verdict, e.g. `verified`. |
| `warning`, `error` | Why attestation is limited, or why the last refresh failed. |

The block is signed with the event, so a receiver can tie each event to the
quote, and therefore the measured boot state, that was current when it was
captured.

## Batch signatures

Hardware keys sign slowly (tens of signatures per second), so a producer MAY
sign a batch of events at once. Each event in the batch then carries
`provenance.batch` instead of `provenance.signature`:

```json
"batch": {
  "statement": {
    "type": "udm.merkle-batch.v1",
    "batch_id": "b0000000000000000000000000000001",
    "root": "319f…(64 hex)",
    "leaf_count": 3,
    "first_sequence": 41,
    "last_sequence": 43,
    "boot_epoch": "0123456789abcdef0123456789abcdef",
    "key_id": "k1",
    "signed_at": "2026-10-29T12:00:03.500000+00:00"
  },
  "signature": "…",
  "leaf_index": 1,
  "proof": ["…", "…"]
}
```

- **Leaf payload:** the event's [signing payload](#signing-payload) with
  `provenance.batch` removed as well.
- **Hashes** (RFC 9162 §2.1): leaf = `SHA-256(0x00 ‖ leaf payload)`,
  node = `SHA-256(0x01 ‖ left ‖ right)`; a tree of *n* leaves splits at the
  largest power of two smaller than *n*.
- **`statement`:** `type` (`udm.merkle-batch.v1`), the `root` (lowercase
  hex), `leaf_count`, and the batch's sequence range, boot epoch, key id and
  signing time.
- **`signature`:** over the canonical JSON of `statement`, with the same
  algorithms and encodings as single events.
- **`proof`:** the RFC 9162 audit path from this leaf to the root, leaf
  first, lowercase hex.

A verifier recomputes the leaf hash, folds in the proof to reach the root,
checks the root equals `statement.root`, and verifies `signature` over the
statement. Each event remains independently verifiable.
[`conformance/vectors/merkle-batch.json`](../conformance/vectors/merkle-batch.json)
holds reference trees and a complete batch.

## Heartbeats

A producer MAY send `system_heartbeat` events at a fixed interval so a
receiver can tell a silent device from a missing one:

```json
{
  "event_type": "system_heartbeat",
  "heartbeat": {"interval_sec": 30, "agent_id": "agent-amr-001", "agent_version": "0.3.1"}
}
```

`heartbeat.interval_sec` (≥ 1) is the promised interval. A receiver SHOULD
treat a source as overdue after missing heartbeats for a few intervals (3 ×
`interval_sec` is a reasonable default), and record the gap. Heartbeats
SHOULD be signed like any other event, so that a removed or bypassed producer
cannot be imitated.

## Receiver-side metadata

Receivers often add their own chain-of-custody data, such as ingest time,
a content hash and a link to the previous event's hash. Those values are
assigned by the receiver, are not signed by the producer, and SHOULD be kept
outside the producer's `provenance` block (or in a separate store), so that
they do not change the signing payload.
