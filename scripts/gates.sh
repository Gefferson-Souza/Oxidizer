#!/bin/sh
# Tyrus quality gates — single source of truth.
#
# This is the ONLY place gate commands are defined. Both `scripts/pre-commit`
# (run by the local git hook) and `.github/workflows/ci.yml` (run in CI)
# invoke this script. Drift between local and CI is therefore architecturally
# impossible.
#
# Rule 9 of docs/standards/POWER_OF_TEN.md (Local-First Validation Parity)
# mandates this single-source design. Adopted via ADR 0008.
#
# Usage:
#   ./scripts/gates.sh             # run all gates
#   ./scripts/gates.sh all         # same as above
#   ./scripts/gates.sh fmt         # cargo fmt --check
#   ./scripts/gates.sh clippy      # cargo clippy --workspace --all-targets -D warnings
#   ./scripts/gates.sh test        # cargo nextest run --workspace --all-targets
#   ./scripts/gates.sh coverage    # cargo llvm-cov nextest --workspace --fail-under-lines <threshold>
#   ./scripts/gates.sh deny        # cargo deny check
#   ./scripts/gates.sh audit       # cargo audit --deny warnings
#
# Environment variables:
#   TYRUS_SKIP_DENY=1      Skip cargo deny check (use only when tool unavailable).
#   TYRUS_SKIP_AUDIT=1     Skip cargo audit (use only when tool unavailable).
#   TYRUS_SKIP_COVERAGE=1  Skip cargo llvm-cov coverage check (e.g. fast pre-commit path).
#   TYRUS_COVERAGE_MIN     Override the minimum line-coverage %
#                          (default: 73 — the 2026-05-03 baseline floor;
#                           ramp-up to 80 tracked in issue #163).
#
# Exit code: 0 on all gates pass, non-zero on first failure.

set -e

# --- Gate definitions --------------------------------------------------------
#
# Flags are intentionally identical between local hook and CI. If you change
# one of these, the change applies to BOTH simultaneously — preventing the
# silent divergence that allowed PR #142 to merge with expect_used violations.

gate_fmt() {
    echo "=== gate: fmt ==="
    cargo fmt --all -- --check
}

gate_clippy() {
    echo "=== gate: clippy ==="
    # --all-targets is critical: it lints test code too. Without this flag,
    # `#[cfg(test)] mod` blocks bypass the strict lint rules entirely.
    cargo clippy --workspace --all-targets -- -D warnings
}

gate_test() {
    echo "=== gate: test ==="
    # nextest by default runs lib tests, integration tests, and doctests.
    # We deliberately do NOT pass --all-targets here: it would try to run
    # benchmarks as test binaries, but bench/runtime_comparison emits a
    # custom report format that does not match libtest's enumeration
    # protocol. clippy's --all-targets gate already lints the bench code;
    # actually executing benches belongs to the dedicated bench job.
    cargo nextest run --workspace
}

gate_coverage() {
    if [ "${TYRUS_SKIP_COVERAGE:-0}" = "1" ]; then
        echo "=== gate: coverage (SKIPPED via TYRUS_SKIP_COVERAGE=1) ==="
        return 0
    fi
    echo "=== gate: coverage ==="
    if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
        echo "ERROR: cargo-llvm-cov is not installed."
        echo "Install with: cargo install cargo-llvm-cov"
        echo "Or skip this gate locally with: TYRUS_SKIP_COVERAGE=1"
        return 1
    fi
    # Rule 5 enforcement (POWER_OF_TEN.md): workspace line coverage
    # threshold. Configurable via TYRUS_COVERAGE_MIN (default tracks the
    # 2026-05-03 baseline floor — see issue #163 for ramp-up to 80%).
    # Benchmarks and `tyrus_test_utils` are excluded from the
    # denominator. The `--ignore-filename-regex` matches against the
    # *absolute path* of each source file (NOT the relative path
    # displayed in the report), so each fragment is anchored on `/`
    # rather than on a leading `^`.
    #
    # Multi-step invocation rationale: integration tests in
    # `tests/src/cli.rs` use `assert_cmd::Command::cargo_bin("tyrus")`,
    # which expects the `tyrus` binary inside
    # `target/llvm-cov-target/debug/`. A plain `cargo llvm-cov nextest`
    # builds test binaries but not the bin — CLI tests then fail with
    # NotFoundError. Splitting the run into
    # (1) clean profraw,
    # (2) `--no-report run --bin tyrus` to populate the binary,
    # (3) `--no-report nextest --workspace` for tests, and
    # (4) a final `report` for threshold enforcement
    # keeps both worlds consistent.
    threshold="${TYRUS_COVERAGE_MIN:-73}"
    # `--profraw-only` is incompatible with `--workspace` (cargo-llvm-cov
    # warns and ignores the latter). Cleaning at the workspace target
    # is implicit, so `--profraw-only` alone is correct.
    cargo llvm-cov clean --profraw-only
    cargo llvm-cov --no-report run --bin tyrus -- --version > /dev/null
    cargo llvm-cov --no-report nextest --workspace
    # `--summary-only` only takes effect with structured output
    # (--json/--lcov/--cobertura). For the human-readable text report
    # we rely on `--fail-under-lines` for the gate signal; the per-file
    # table is informative for the contributor and forwarded by CI.
    cargo llvm-cov report \
        --fail-under-lines "$threshold" \
        --ignore-filename-regex '/tyrus_test_utils/|/benches/'
}

gate_deny() {
    if [ "${TYRUS_SKIP_DENY:-0}" = "1" ]; then
        echo "=== gate: deny (SKIPPED via TYRUS_SKIP_DENY=1) ==="
        return 0
    fi
    echo "=== gate: deny ==="
    cargo deny check
}

gate_audit() {
    if [ "${TYRUS_SKIP_AUDIT:-0}" = "1" ]; then
        echo "=== gate: audit (SKIPPED via TYRUS_SKIP_AUDIT=1) ==="
        return 0
    fi
    echo "=== gate: audit ==="
    # Ignored advisories are duplicated from deny.toml [advisories].ignore
    # because cargo-audit doesn't read deny.toml. Keep these in sync with
    # deny.toml whenever advisories are triaged.
    cargo audit \
        --ignore RUSTSEC-2025-0119 \
        --ignore RUSTSEC-2026-0097 \
        --deny warnings
}

# --- Dispatcher --------------------------------------------------------------

cmd="${1:-all}"
case "$cmd" in
    fmt)      gate_fmt ;;
    clippy)   gate_clippy ;;
    test)     gate_test ;;
    coverage) gate_coverage ;;
    deny)     gate_deny ;;
    audit)    gate_audit ;;
    all)
        gate_fmt
        gate_clippy
        gate_test
        gate_coverage
        gate_deny
        gate_audit
        echo ""
        echo "=== ALL GATES PASSED ==="
        ;;
    -h|--help|help)
        sed -n '2,25p' "$0"
        exit 0
        ;;
    *)
        echo "ERROR: unknown gate '$cmd'" >&2
        echo "Run: $0 help" >&2
        exit 2
        ;;
esac
