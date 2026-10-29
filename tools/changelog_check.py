#!/usr/bin/env python3
"""Enforce CHANGELOG.md updates when ``schemas/`` or ``spec/`` change.

Compares the working tree against a base ref (default ``origin/main``) and
fails if any path under ``schemas/`` or ``spec/`` was modified WITHOUT
``CHANGELOG.md`` also being modified.

Usage::

    python3 tools/changelog_check.py                # diff against origin/main
    python3 tools/changelog_check.py --base HEAD~1  # diff against previous commit
    python3 tools/changelog_check.py --files a b c  # check an explicit file list

Exit codes:
    0  no schema/spec changes, OR changelog updated alongside them
    1  schema/spec changed but CHANGELOG.md untouched
    2  invocation/git error
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path

WATCHED_PREFIXES = ("schemas/", "spec/")
CHANGELOG_PATH = "CHANGELOG.md"


def _git_changed_files(base: str) -> list[str]:
    """Return the list of file paths changed between ``base`` and HEAD,
    plus any uncommitted changes in the working tree."""
    try:
        committed = subprocess.check_output(
            ["git", "diff", "--name-only", f"{base}...HEAD"],
            text=True, stderr=subprocess.PIPE,
        ).splitlines()
    except subprocess.CalledProcessError as exc:
        sys.stderr.write(f"ERROR: git diff failed: {exc.stderr}\n")
        raise SystemExit(2)

    try:
        uncommitted = subprocess.check_output(
            ["git", "status", "--porcelain"],
            text=True, stderr=subprocess.PIPE,
        ).splitlines()
    except subprocess.CalledProcessError as exc:
        sys.stderr.write(f"ERROR: git status failed: {exc.stderr}\n")
        raise SystemExit(2)

    files = list(committed)
    for line in uncommitted:
        # `git status --porcelain` lines: "XY path" or "XY path -> path".
        path = line[3:].strip()
        if " -> " in path:
            path = path.split(" -> ", 1)[1]
        if path:
            files.append(path)
    return sorted(set(files))


def _is_watched(path: str) -> bool:
    return any(path.startswith(prefix) for prefix in WATCHED_PREFIXES)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--base",
        default=os.environ.get("UDM_CHANGELOG_BASE", "origin/main"),
        help="Git ref to diff against (default: origin/main).",
    )
    parser.add_argument(
        "--files",
        nargs="+",
        help="Explicit file list to check (skips git).",
    )
    args = parser.parse_args(argv)

    if args.files is not None:
        changed = sorted(args.files)
    else:
        changed = _git_changed_files(args.base)

    watched = [p for p in changed if _is_watched(p)]
    if not watched:
        print("changelog-check: no schema/spec changes detected — OK")
        return 0

    if CHANGELOG_PATH in changed:
        print(
            "changelog-check: "
            f"{len(watched)} watched file(s) changed; CHANGELOG.md updated — OK"
        )
        return 0

    sys.stderr.write(
        "changelog-check: FAIL\n"
        f"  The following watched files changed but {CHANGELOG_PATH} was not "
        "updated:\n"
    )
    for p in watched:
        sys.stderr.write(f"    - {p}\n")
    sys.stderr.write(
        f"\n  Add an entry to {CHANGELOG_PATH} under [Unreleased] describing the "
        "change.\n"
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
