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
#   ./scripts/gates.sh           # run all gates
#   ./scripts/gates.sh all       # same as above
#   ./scripts/gates.sh fmt       # cargo fmt --check
#   ./scripts/gates.sh clippy    # cargo clippy --workspace --all-targets -D warnings
#   ./scripts/gates.sh test      # cargo nextest run --workspace --all-targets
#   ./scripts/gates.sh deny      # cargo deny check
#   ./scripts/gates.sh audit     # cargo audit --deny warnings
#
# Environment variables:
#   TYRUS_SKIP_DENY=1   Skip cargo deny check (use only when tool unavailable).
#   TYRUS_SKIP_AUDIT=1  Skip cargo audit (use only when tool unavailable).
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
    fmt)    gate_fmt ;;
    clippy) gate_clippy ;;
    test)   gate_test ;;
    deny)   gate_deny ;;
    audit)  gate_audit ;;
    all)
        gate_fmt
        gate_clippy
        gate_test
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
