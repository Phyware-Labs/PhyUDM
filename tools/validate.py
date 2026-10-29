#!/usr/bin/env python3
"""Validate UDM JSON payloads against the canonical UDM JSON Schema.

Usage::

    python3 tools/validate.py path/to/event.json [more.json ...]
    python3 tools/validate.py --schema-version 0.0.3 path/to/event.json
    python3 tools/validate.py --conformance         # run the full suite

Exit status: ``0`` if every input validates, ``1`` otherwise. Errors are
written to stderr with the failing JSON Pointer + a short message.

This tool is intentionally dependency-light: it requires the ``jsonschema``
package and reads the schemas straight from ``schemas/v<version>/``.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Iterable

try:
    from jsonschema import Draft202012Validator
    from jsonschema import exceptions as js_exc
    from referencing import Registry, Resource
    from referencing.jsonschema import DRAFT202012
except ImportError as exc:  # pragma: no cover - dependency hint
    sys.stderr.write(
        f"ERROR: required dependencies missing: {exc}\n"
        "Install with: pip install 'jsonschema>=4.18' 'referencing>=0.30'\n"
    )
    sys.exit(2)


REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_VERSION = "0.0.3"


def _load_schema(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as fh:
        return json.load(fh)


def _build_registry(schema_dir: Path) -> Registry:
    """Register every schema in ``schema_dir`` under its ``$id`` URL and a
    filesystem-relative URL so internal ``$ref``s resolve."""
    resources: list[tuple[str, Resource]] = []
    for path in sorted(schema_dir.rglob("*.schema.json")):
        # Skip macOS resource-fork sidecars (``._*``).
        if path.name.startswith("._"):
            continue
        schema = _load_schema(path)
        resource = Resource(contents=schema, specification=DRAFT202012)
        # Register by canonical $id.
        if "$id" in schema:
            resources.append((schema["$id"], resource))
        # Register by relative filename to support local $ref "envelope.schema.json".
        rel = path.relative_to(schema_dir).as_posix()
        resources.append((rel, resource))
        # Register by basename (e.g. "envelope.schema.json") so refs at the
        # same level resolve via base URI lookup.
        resources.append((path.name, resource))
    return Registry().with_resources(resources)


def _load_event_validator(version: str) -> Draft202012Validator:
    schema_dir = REPO_ROOT / "schemas" / f"v{version}"
    if not schema_dir.is_dir():
        raise SystemExit(f"ERROR: schemas/v{version}/ not found")
    event_schema = _load_schema(schema_dir / "event.schema.json")
    registry = _build_registry(schema_dir)
    return Draft202012Validator(
        schema=event_schema,
        registry=registry,
        format_checker=Draft202012Validator.FORMAT_CHECKER,
    )


def _format_error(err: js_exc.ValidationError) -> str:
    location = "/".join(str(p) for p in err.absolute_path) or "<root>"
    keyword = err.validator
    return f"  - {location} [{keyword}]: {err.message}"


def _validate_file(
    validator: Draft202012Validator,
    path: Path,
) -> tuple[bool, list[str]]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        return False, [f"  - <parse>: {exc.msg} (line {exc.lineno}, col {exc.colno})"]
    errors = sorted(validator.iter_errors(payload), key=lambda e: list(e.absolute_path))
    if not errors:
        return True, []
    return False, [_format_error(e) for e in errors]


def _iter_conformance(kind: str) -> Iterable[Path]:
    base = REPO_ROOT / "conformance" / kind
    if not base.is_dir():
        return []
    return sorted(p for p in base.glob("*.json"))


def _run_conformance(validator: Draft202012Validator) -> int:
    overall_ok = True
    # valid/ + edge/ must all PASS.
    for kind in ("valid", "edge"):
        for path in _iter_conformance(kind):
            ok, errs = _validate_file(validator, path)
            label = f"[{kind:>7}] {path.relative_to(REPO_ROOT)}"
            if ok:
                print(f"PASS  {label}")
            else:
                overall_ok = False
                print(f"FAIL  {label}", file=sys.stderr)
                for e in errs:
                    print(e, file=sys.stderr)
    # invalid/ must FAIL (at least one error).
    for path in _iter_conformance("invalid"):
        ok, _errs = _validate_file(validator, path)
        label = f"[invalid] {path.relative_to(REPO_ROOT)}"
        if not ok:
            print(f"PASS  {label} (expected failure)")
        else:
            overall_ok = False
            print(
                f"FAIL  {label} (expected failure, but validated OK)",
                file=sys.stderr,
            )
    # legacy/ is informational only — fixtures here are pre-canonical and the
    # drift between them and the schema is tracked in each meta.yaml. We DO
    # NOT gate the suite on legacy outcomes.
    for path in _iter_conformance("legacy"):
        ok, errs = _validate_file(validator, path)
        label = f"[ legacy] {path.relative_to(REPO_ROOT)}"
        if ok:
            print(f"INFO  {label} validates (drift may have been resolved)")
        else:
            print(f"DRIFT {label} (expected; see {path.stem}.meta.yaml)")
    return 0 if overall_ok else 1


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Validate UDM JSON against the canonical UDM JSON Schema.",
    )
    parser.add_argument(
        "files",
        nargs="*",
        type=Path,
        help="JSON files to validate against the event schema.",
    )
    parser.add_argument(
        "--schema-version",
        default=DEFAULT_VERSION,
        help=f"UDM schema version directory under schemas/ (default: {DEFAULT_VERSION}).",
    )
    parser.add_argument(
        "--conformance",
        action="store_true",
        help="Run the full conformance suite (valid/, invalid/, edge/, legacy/).",
    )
    args = parser.parse_args(argv)

    validator = _load_event_validator(args.schema_version)

    if args.conformance:
        return _run_conformance(validator)

    if not args.files:
        parser.error("provide one or more JSON files, or pass --conformance")

    overall_ok = True
    for path in args.files:
        ok, errs = _validate_file(validator, path)
        if ok:
            print(f"PASS  {path}")
        else:
            overall_ok = False
            print(f"FAIL  {path}", file=sys.stderr)
            for e in errs:
                print(e, file=sys.stderr)
    return 0 if overall_ok else 1


if __name__ == "__main__":
    sys.exit(main())
