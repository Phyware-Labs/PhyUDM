#!/usr/bin/env python3
"""Generate the signing test vectors in ``conformance/vectors/``.

Reference implementation of the signing rules in ``spec/provenance.md``,
standard library only, so producers and verifiers in any language can check
themselves against the same bytes:

* ``provenance-signing.json``: events, their signing payloads, and the
  HMAC-SHA256 of each payload under the vector's key. The payload rules are
  the same for every algorithm; HMAC keeps the vectors reproducible
  without an elliptic-curve library.
* ``merkle-batch.json``: RFC 9162 Merkle trees (roots and audit paths for
  1 to 7 leaves) and one complete ``udm.merkle-batch.v1`` batch.

Run from the repo root::

    python3 tools/build_vectors.py
"""

from __future__ import annotations

import hashlib
import hmac
import json
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parent.parent
OUT_DIR = REPO_ROOT / "conformance" / "vectors"
KEY = "udm-test-key"
BATCH_TYPE = "udm.merkle-batch.v1"


# ---------------------------------------------------------------------------
# Signing payload (spec/provenance.md#signing-payload).
# ---------------------------------------------------------------------------

def canonical(value: Any) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def strip_nulls(value: Any) -> Any:
    if isinstance(value, dict):
        return {k: strip_nulls(v) for k, v in value.items() if v is not None}
    if isinstance(value, list):
        return [strip_nulls(v) for v in value]
    return value


def signing_payload(event: dict[str, Any], *, leaf: bool = False) -> str:
    """The bytes a producer signs: the event without nulls and without
    `provenance.signature` (and, for a batch leaf, `provenance.batch`)."""
    body = strip_nulls(event)
    provenance = body.get("provenance")
    if isinstance(provenance, dict):
        provenance.pop("signature", None)
        if leaf:
            provenance.pop("batch", None)
        if not provenance:
            del body["provenance"]
    return canonical(body)


def hmac_hex(key: str, payload: str) -> str:
    return hmac.new(key.encode("utf-8"), payload.encode("utf-8"), hashlib.sha256).hexdigest()


SIGNING_VECTORS: list[dict[str, Any]] = [
    {
        "name": "minimal_envelope",
        "key": KEY,
        "event": {
            "udm_version": "0.0.3",
            "event_id": "0192f8a0-0000-7000-8000-000000000001",
            "event_type": "telemetry_periodic",
            "source_id": "robot-001",
            "source_type": "amr",
            "captured_at": "2026-10-27T10:00:00Z",
        },
    },
    {
        "name": "domains_numbers_and_existing_provenance",
        "key": KEY,
        "event": {
            "udm_version": "0.0.3",
            "event_id": "0192f8a0-0000-7000-8000-000000000002",
            "event_type": "telemetry_periodic",
            "source_id": "robot-002",
            "source_type": "amr",
            "captured_at": "2026-10-27T10:00:00.123456Z",
            "sent_at": "2026-10-27T10:00:00.200000+00:00",
            "sequence_num": 42,
            "session_id": None,
            "location": {"latitude": 51.5072, "longitude": -0.1276, "heading_deg": 270.0},
            "power": {"battery_soc_pct": 87.5, "voltage_v": 48.0, "current_a": -3.25},
            "motion": {"speed_mps": 1.5e-07, "odometer_m": 1e16, "tiny": 1e-05},
            "perception": {"detections": [{"class": "person", "confidence": 0.87}, None]},
            "provenance": {
                "signature": "ignored-by-the-payload",
                "algorithm": "hmac-sha256",
            },
        },
    },
    {
        "name": "provenance_metadata_and_attestation_are_signed",
        "key": KEY,
        "event": {
            "udm_version": "0.0.3",
            "event_id": "0192f8a0-0000-7000-8000-000000000005",
            "event_type": "telemetry_periodic",
            "source_id": "robot-005",
            "source_type": "amr",
            "captured_at": "2026-10-27T10:00:00Z",
            "sent_at": "2026-10-27T10:00:00.050000+00:00",
            "sequence_num": 7,
            "provenance": {
                "agent_id": "agent-0123456789ab",
                "agent_version": "0.1.0",
                "signer": "tpm",
                "key_id": "device-key-1",
                "algorithm": "es256",
                "signed_at": "2026-10-27T10:00:00.050000+00:00",
                "sequence_num": 7,
                "boot_epoch": "5f0c6b1e2d3a4f5e8a9b0c1d2e3f4a5b",
                "boot_counter": 12,
                "device_attestation": {
                    "mode": "tpm",
                    "quote_id": "q-17",
                    "quote_status": "verified",
                },
                "signed_fields": None,
                "signature": "replaced-by-the-real-signature",
            },
        },
    },
    {
        "name": "unicode_escapes_and_nulls_in_extensions",
        "key": "another key with spaces",
        "event": {
            "udm_version": "0.0.3",
            "event_id": "0192f8a0-0000-7000-8000-000000000003",
            "event_type": "safety_violation",
            "source_id": "robot-é-三-🤖",
            "source_type": "humanoid",
            "captured_at": "2026-10-27T10:00:00Z",
            "safety": {"message": "quote \" backslash \\ tab \t newline \n ctrl \u0001 del \u007f"},
            "extensions": {
                "custom_status": "valid",
                "custom_cleared": None,
                "nested": {"keep": [None, 1, "x"], "drop": None},
            },
        },
    },
    {
        "name": "large_and_negative_integers",
        "key": KEY,
        "event": {
            "event_id": "0192f8a0-0000-7000-8000-000000000004",
            "event_type": "telemetry_periodic",
            "source_id": "robot-004",
            "source_type": "amr",
            "captured_at": "2026-10-27T10:00:00Z",
            "sequence_num": 9007199254740993,
            "extensions": {"offset": -17, "zero": 0, "float_zero": 0.0, "neg_zero": -0.0},
        },
    },
    {
        # A block holding only the signature signs like no block at all.
        "name": "provenance_with_only_a_signature",
        "key": KEY,
        "event": {
            "event_id": "0192f8a0-0000-7000-8000-000000000006",
            "event_type": "telemetry_periodic",
            "source_id": "robot-006",
            "source_type": "amr",
            "captured_at": "2026-10-27T10:00:00Z",
            "provenance": {"signature": "abc"},
        },
    },
]


def signing_vectors() -> dict[str, Any]:
    out = []
    for v in SIGNING_VECTORS:
        payload = signing_payload(v["event"])
        out.append({**v, "payload": payload, "hmac_sha256": hmac_hex(v["key"], payload)})
    return {
        "description": (
            "Signing payload vectors (spec/provenance.md#signing-payload). "
            "Generated by tools/build_vectors.py; do not edit by hand."
        ),
        "vectors": out,
    }


# ---------------------------------------------------------------------------
# Merkle batches (spec/provenance.md#batch-signatures).
# ---------------------------------------------------------------------------

def leaf_hash(payload: bytes) -> bytes:
    return hashlib.sha256(b"\x00" + payload).digest()


def node_hash(left: bytes, right: bytes) -> bytes:
    return hashlib.sha256(b"\x01" + left + right).digest()


def split(n: int) -> int:
    k = 1
    while k * 2 < n:
        k *= 2
    return k


def root(leaves: list[bytes]) -> bytes:
    if len(leaves) == 1:
        return leaves[0]
    k = split(len(leaves))
    return node_hash(root(leaves[:k]), root(leaves[k:]))


def audit_path(m: int, leaves: list[bytes]) -> list[bytes]:
    if len(leaves) == 1:
        return []
    k = split(len(leaves))
    if m < k:
        return audit_path(m, leaves[:k]) + [root(leaves[k:])]
    return audit_path(m - k, leaves[k:]) + [root(leaves[:k])]


def trees() -> list[dict[str, Any]]:
    out = []
    for n in range(1, 8):
        payloads = [f"leaf-{i}" for i in range(n)]
        leaves = [leaf_hash(p.encode()) for p in payloads]
        out.append(
            {
                "payloads": payloads,
                "root": root(leaves).hex(),
                "proofs": [[h.hex() for h in audit_path(i, leaves)] for i in range(n)],
            }
        )
    return out


def batch() -> dict[str, Any]:
    events = []
    for i in range(3):
        events.append(
            {
                "udm_version": "0.0.3",
                "event_id": f"0192f8a0-0000-7000-8000-00000000010{i}",
                "event_type": "telemetry_periodic",
                "source_id": "robot-001",
                "source_type": "amr",
                "captured_at": f"2026-10-29T12:00:0{i}Z",
                "location": {"x_m": 1.5 + i, "heading_deg": 90.0},
                "provenance": {
                    "agent_id": "agent-000000000001",
                    "agent_version": "0.1.0",
                    "signed_at": "2026-10-29T12:00:03.500000+00:00",
                    "sequence_num": 41 + i,
                    "boot_epoch": "0123456789abcdef0123456789abcdef",
                    "algorithm": "hmac-sha256",
                    "signer": "hmac",
                    "key_id": "k1",
                },
            }
        )
    leaves = [leaf_hash(signing_payload(e, leaf=True).encode()) for e in events]
    statement = {
        "type": BATCH_TYPE,
        "batch_id": "b0000000000000000000000000000001",
        "root": root(leaves).hex(),
        "leaf_count": len(events),
        "first_sequence": 41,
        "last_sequence": 43,
        "boot_epoch": "0123456789abcdef0123456789abcdef",
        "key_id": "k1",
        "signed_at": "2026-10-29T12:00:03.500000+00:00",
    }
    signature = hmac_hex(KEY, canonical(statement))
    for i, event in enumerate(events):
        event["provenance"]["batch"] = {
            "statement": statement,
            "signature": signature,
            "leaf_index": i,
            "proof": [h.hex() for h in audit_path(i, leaves)],
        }
    return {
        "key": KEY,
        "statement_payload": canonical(statement),
        "leaf_payloads": [signing_payload(e, leaf=True) for e in events],
        "events": events,
    }


def merkle_vectors() -> dict[str, Any]:
    return {
        "description": (
            "Merkle batch vectors (spec/provenance.md#batch-signatures). "
            "Generated by tools/build_vectors.py; do not edit by hand."
        ),
        "trees": trees(),
        "batch": batch(),
    }


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    for name, doc in [
        ("provenance-signing.json", signing_vectors()),
        ("merkle-batch.json", merkle_vectors()),
    ]:
        (OUT_DIR / name).write_text(
            json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
    print(f"Wrote 2 vector files to {OUT_DIR}/")


if __name__ == "__main__":
    main()
