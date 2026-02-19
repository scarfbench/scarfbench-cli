SHELL := /bin/bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c

CARGO ?= cargo
RUSTUP ?= rustup

WORKSPACE ?= --workspace
FEATURES ?=

NEXTTEST_ARGS ?=
LLVM_COV_ARGS ?=

DIST_VERSION ?= 0.30.4

# Optional tools (developer goodies)
AUDIT_ARGS ?=
DENY_ARGS ?=
OUTDATED_ARGS ?=
EXPAND_ARGS ?=
WATCH_ARGS ?=

.PHONY: help default all setup check-rustup check-cargo ensure-components ensure-llvm-tools \
        ensure-nextest ensure-llvm-cov ensure-dist \
        ensure-audit ensure-deny ensure-outdated ensure-expand ensure-watch ensure-dev-tools \
        fmt clippy build test coverage clean \
        audit deny outdated expand watch \
        dist-plan dist-build dist-host dist-init

help:
	@echo "Usage:"
	@echo "  make [target] [VAR=value]"
	@echo ""
	@echo "Default target:"
	@echo "  all        Run full pipeline (setup → fmt → clippy → build → test → coverage)"
	@echo ""
	@echo "Targets:"
	@echo "  setup      Check/install rustup, cargo, components, nextest, llvm-cov, dist"
	@echo "  dev-tools  Install optional developer tools (audit/deny/outdated/expand/watch)"
	@echo "  fmt        Run cargo fmt --all"
	@echo "  clippy     Run cargo clippy with warnings denied"
	@echo "  build      Run cargo build"
	@echo "  test       Run tests using cargo nextest"
	@echo "  coverage   Run coverage using cargo llvm-cov + nextest"
	@echo "  audit      Run cargo audit (requires cargo-audit)"
	@echo "  deny       Run cargo deny check (requires cargo-deny)"
	@echo "  outdated   Run cargo outdated (requires cargo-outdated)"
	@echo "  expand     Run cargo expand (requires cargo-expand)"
	@echo "  watch      Run cargo watch (requires cargo-watch)"
	@echo "  dist-*     Release helper commands (dist init/plan/build/host)"
	@echo "  clean      Run cargo clean"
	@echo "  help       Show this help message"
	@echo ""
	@echo "Variables:"
	@echo "  WORKSPACE=--workspace        Target whole workspace (set empty for single crate)"
	@echo "  FEATURES=...                 Extra cargo flags (e.g. --all-features or --features foo)"
	@echo "  NEXTTEST_ARGS=...            Extra args passed to cargo nextest run"
	@echo "  LLVM_COV_ARGS=...            Extra args passed to cargo llvm-cov"
	@echo "  DIST_VERSION=...             cargo-dist version to install (default: $(DIST_VERSION))"
	@echo "  CARGO=...                    Override cargo binary (default: cargo)"
	@echo "  RUSTUP=...                   Override rustup binary (default: rustup)"
	@echo ""
	@echo "Examples:"
	@echo "  make"
	@echo "  make test NEXTTEST_ARGS=\"--nocapture\""
	@echo "  make coverage LLVM_COV_ARGS=\"--fail-under-lines 80\""
	@echo "  make dev-tools"
	@echo "  make audit"
	@echo "  make dist-plan TAG=v0.1.0"

default: all

all: setup fmt clippy build coverage
	@echo "[OK] Cargo build pipeline complete."

setup: check-rustup check-cargo ensure-components ensure-llvm-tools ensure-nextest ensure-llvm-cov ensure-dist
	@echo "[OK] Toolchain and required tools are ready."

dev-tools: check-cargo ensure-audit ensure-deny ensure-outdated ensure-expand ensure-watch
	@echo "[OK] Optional developer tools are ready."

check-rustup:
	@if command -v $(RUSTUP) >/dev/null 2>&1; then \
	  echo "[OK] rustup found: $$($(RUSTUP) --version | head -n1)"; \
	else \
	  echo "[ERROR] rustup not found."; \
	  echo "Install rustup (includes cargo) from:"; \
	  echo "  https://rustup.rs"; \
	  echo "Then re-run: make setup"; \
	  exit 1; \
	fi

check-cargo:
	@if command -v $(CARGO) >/dev/null 2>&1; then \
	  echo "[OK] cargo found: $$($(CARGO) --version | head -n1)"; \
	else \
	  echo "[ERROR] cargo not found."; \
	  echo "If you installed rustup, ensure ~/.cargo/bin is on PATH."; \
	  echo "Install rustup from https://rustup.rs (cargo is included)."; \
	  echo "Then re-run: make setup"; \
	  exit 1; \
	fi

ensure-components: check-rustup
	@echo "[INFO] Ensuring rustfmt + clippy components via rustup..."
	@$(RUSTUP) component add rustfmt clippy >/dev/null
	@echo "[OK] rustfmt + clippy installed."

ensure-llvm-tools: check-rustup
	@echo "[INFO] Ensuring llvm-tools-preview via rustup (required for coverage)..."
	@$(RUSTUP) component add llvm-tools-preview >/dev/null
	@echo "[OK] llvm-tools-preview installed."

ensure-nextest: check-cargo
	@if command -v cargo-nextest >/dev/null 2>&1; then \
	  echo "[OK] nextest found: $$($(CARGO) nextest --version | head -n1)"; \
	else \
	  echo "[INFO] Installing nextest (cargo-nextest) ..."; \
	  $(CARGO) install cargo-nextest --locked; \
	  echo "[OK] nextest installed."; \
	fi

ensure-llvm-cov: check-cargo
	@if command -v cargo-llvm-cov >/dev/null 2>&1; then \
	  echo "[OK] llvm-cov found: $$($(CARGO) llvm-cov --version | head -n1)"; \
	else \
	  echo "[INFO] Installing llvm-cov (cargo-llvm-cov) ..."; \
	  $(CARGO) install cargo-llvm-cov; \
	  echo "[OK] llvm-cov installed."; \
	fi

ensure-dist: check-cargo
	@if command -v dist >/dev/null 2>&1; then \
	  echo "[OK] dist found: $$(dist --version | head -n1)"; \
	else \
	  echo "[INFO] Installing dist (cargo-dist) $(DIST_VERSION) ..."; \
	  $(CARGO) install cargo-dist --version $(DIST_VERSION) --locked; \
	  echo "[OK] dist installed."; \
	fi

# Optional developer goodies ----------------------------------------------------

ensure-audit: check-cargo
	@if command -v cargo-audit >/dev/null 2>&1; then \
	  echo "[OK] cargo-audit found: $$(cargo audit --version | head -n1)"; \
	else \
	  echo "[INFO] Installing cargo-audit ..."; \
	  $(CARGO) install cargo-audit --locked; \
	  echo "[OK] cargo-audit installed."; \
	fi

ensure-deny: check-cargo
	@if command -v cargo-deny >/dev/null 2>&1; then \
	  echo "[OK] cargo-deny found: $$(cargo deny --version | head -n1)"; \
	else \
	  echo "[INFO] Installing cargo-deny ..."; \
	  $(CARGO) install cargo-deny --locked; \
	  echo "[OK] cargo-deny installed."; \
	fi

ensure-outdated: check-cargo
	@if command -v cargo-outdated >/dev/null 2>&1; then \
	  echo "[OK] cargo-outdated found: $$(cargo outdated --version | head -n1)"; \
	else \
	  echo "[INFO] Installing cargo-outdated ..."; \
	  $(CARGO) install cargo-outdated --locked; \
	  echo "[OK] cargo-outdated installed."; \
	fi

ensure-expand: check-cargo
	@if command -v cargo-expand >/dev/null 2>&1; then \
	  echo "[OK] cargo-expand found: $$(cargo expand --version 2>/dev/null | head -n1 || echo 'installed')"; \
	else \
	  echo "[INFO] Installing cargo-expand ..."; \
	  $(CARGO) install cargo-expand --locked; \
	  echo "[OK] cargo-expand installed."; \
	fi

ensure-watch: check-cargo
	@if command -v cargo-watch >/dev/null 2>&1; then \
	  echo "[OK] cargo-watch found: $$(cargo watch --version | head -n1)"; \
	else \
	  echo "[INFO] Installing cargo-watch ..."; \
	  $(CARGO) install cargo-watch --locked; \
	  echo "[OK] cargo-watch installed."; \
	fi

# Core workflow ----------------------------------------------------------------

fmt: check-cargo ensure-components
	@echo "[INFO] cargo fmt --all"
	@$(CARGO) fmt --all

clippy: check-cargo ensure-components
	@echo "[INFO] cargo clippy (deny warnings)"
	@$(CARGO) clippy $(WORKSPACE) $(FEATURES) -- -D warnings

build: check-cargo
	@echo "[INFO] cargo build"
	@$(CARGO) build $(WORKSPACE) $(FEATURES)

test: check-cargo ensure-nextest
	@echo "[INFO] cargo nextest run"
	@$(CARGO) nextest run $(WORKSPACE) $(FEATURES) $(NEXTTEST_ARGS)

coverage: check-cargo ensure-llvm-tools ensure-llvm-cov ensure-nextest
	@echo "[INFO] cargo llvm-cov nextest"
	@$(CARGO) llvm-cov nextest $(WORKSPACE) $(FEATURES) $(LLVM_COV_ARGS)
	@echo "[OK] Coverage HTML report generated under target/llvm-cov/html/"

clean: check-cargo
	@$(CARGO) clean

# Optional tool commands --------------------------------------------------------

audit: check-cargo ensure-audit
	@echo "[INFO] cargo audit"
	@$(CARGO) audit $(AUDIT_ARGS)

deny: check-cargo ensure-deny
	@echo "[INFO] cargo deny check"
	@$(CARGO) deny check $(DENY_ARGS)

outdated: check-cargo ensure-outdated
	@echo "[INFO] cargo outdated"
	@$(CARGO) outdated $(OUTDATED_ARGS)

expand: check-cargo ensure-expand
	@echo "[INFO] cargo expand $(EXPAND_ARGS)"
	@$(CARGO) expand $(EXPAND_ARGS)

watch: check-cargo ensure-watch
	@echo "[INFO] cargo watch $(WATCH_ARGS)"
	@$(CARGO) watch $(WATCH_ARGS)

# These are my convenience targets; dist requires a tag/version to operate on.
TAG ?=

#   make dist-plan TAG=v0.1.1
dist-init: check-cargo ensure-dist
	@echo "[INFO] dist init"
	@dist init

dist-plan: check-cargo ensure-dist
	@if [ -z "$(TAG)" ]; then \
	  echo "[ERROR] TAG is required (e.g., TAG=v0.1.1)"; \
	  exit 1; \
	fi
	@echo "[INFO] dist plan --tag=$(TAG)"
	@dist plan --tag="$(TAG)"

#   make dist-build TAG=v0.1.1
dist-build: check-cargo ensure-dist
	@if [ -z "$(TAG)" ]; then \
	  echo "[ERROR] TAG is required (e.g., TAG=v0.1.1)"; \
	  exit 1; \
	fi
	@echo "[INFO] dist build --tag=$(TAG)"
	@dist build --tag="$(TAG)"
