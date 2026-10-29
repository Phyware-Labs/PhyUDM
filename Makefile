# UDM Toolchain Makefile
#
# Top-level targets used by contributors and CI to build, validate, and
# regenerate the canonical schemas + conformance suite.

PYTHON ?= python3
PIP ?= $(PYTHON) -m pip
CARGO ?= cargo
SCHEMA_VERSION ?= 0.0.3

CHANGELOG_BASE ?= origin/main

.PHONY: help install build-schemas build-conformance build validate \
        validate-fixture conformance changelog-check check clean \
        rust-fmt rust-clippy rust-test rust-build rust-check rust-all

help:  ## Show this help.
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  %-22s %s\n", $$1, $$2}'

install:  ## Install Python dependencies for tools/.
	$(PIP) install --quiet \
		'jsonschema>=4.18' 'referencing>=0.30' 'rfc3339-validator>=0.1' \
		'PyYAML>=6'

build-schemas:  ## Regenerate schemas/v$(SCHEMA_VERSION)/*.json from tools/build_schemas.py.
	$(PYTHON) tools/build_schemas.py

build-conformance:  ## Regenerate conformance/{valid,invalid,edge,legacy}/*.
	$(PYTHON) tools/build_conformance.py

build: build-schemas build-conformance  ## Regenerate all generated artifacts.

validate:  ## Run the full conformance suite against schemas/v$(SCHEMA_VERSION).
	$(PYTHON) tools/validate.py --conformance --schema-version $(SCHEMA_VERSION)

validate-fixture:  ## Validate one or more files. Use ARGS="path/to/event.json".
	$(PYTHON) tools/validate.py --schema-version $(SCHEMA_VERSION) $(ARGS)

conformance: validate  ## Alias for `validate` (run the conformance suite).

changelog-check:  ## Fail if schemas/ or spec/ changed without updating CHANGELOG.md.
	$(PYTHON) tools/changelog_check.py --base $(CHANGELOG_BASE)

check: validate changelog-check rust-check  ## Run all gating checks.

# -------- Rust workspace (crates/*) -------------------------------------

rust-fmt:  ## Check Rust formatting (cargo fmt --check).
	$(CARGO) fmt --all -- --check

rust-clippy:  ## Run clippy with -D warnings across the workspace.
	$(CARGO) clippy --workspace --all-targets -- -D warnings

rust-test:  ## Run the Rust workspace test suite.
	$(CARGO) test --workspace --all-targets

rust-build:  ## Build the Rust workspace.
	$(CARGO) build --workspace

rust-check: rust-fmt rust-clippy rust-test  ## Run fmt + clippy + test for the Rust workspace.

rust-all: rust-build rust-check  ## Build + fmt + clippy + test.

clean:  ## Remove caches.
	find . -name __pycache__ -type d -prune -exec rm -rf {} +
	find . -name '._*' -delete
	$(CARGO) clean
