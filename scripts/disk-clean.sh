#!/usr/bin/env bash
#
# Tyrus disk hygiene.
#
# Usage:
#   bash scripts/disk-clean.sh             # incremental: drop stale artifacts only
#   bash scripts/disk-clean.sh --all       # full wipe: cargo clean + remove shared targets
#   bash scripts/disk-clean.sh --report    # report current footprint without deleting
#
# The shared `/tmp/tyrus_test_target` is intentional — it lets every
# `assert_rust_compiles` and `assert_output_equivalent` test reuse compiled
# dependencies (axum, tokio, serde, reqwest, ...) instead of rebuilding each
# crate per test. Without it, the test suite is ≈10× slower and drowns CI.
# The trade-off is unbounded growth, hence this script.

set -euo pipefail

mode="${1:-incremental}"
repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
shared_target="${TMPDIR:-/tmp}/tyrus_test_target"

report() {
    echo "=== Tyrus disk footprint ==="
    du -sh "$repo_root/target" 2>/dev/null || echo "(no repo target)"
    du -sh "$repo_root/tests/target" 2>/dev/null || true
    du -sh "$shared_target" 2>/dev/null || echo "(no shared test target)"
    du -sh "$repo_root/benchmarks/academic" 2>/dev/null || true
    echo "Repo total: $(du -sh "$repo_root" | cut -f1)"
}

case "$mode" in
    --report)
        report
        ;;
    --all)
        echo "Full wipe — recovering all build artifacts..."
        (cd "$repo_root" && cargo clean) || true
        (cd "$repo_root" && cargo llvm-cov clean --workspace 2>/dev/null) || true
        rm -rf "$shared_target" "$repo_root/tests/target"
        echo "Done."
        report
        ;;
    incremental|"")
        echo "Incremental cleanup — dropping stale artifacts only..."
        # Stale .profraw files (coverage instrumentation) > 7 days
        find "$repo_root/target/llvm-cov-target" -name '*.profraw' -atime +7 -delete 2>/dev/null || true
        # Test target files unused for > 14 days
        find "$shared_target" -type f -atime +14 -delete 2>/dev/null || true
        # Cargo's own incremental build cache > 30 days
        find "$repo_root/target/debug/incremental" -mindepth 1 -maxdepth 1 -atime +30 -exec rm -rf {} + 2>/dev/null || true
        echo "Done."
        report
        ;;
    *)
        echo "ERROR: unknown mode '$mode'" >&2
        echo "Usage: $0 [--all|--report|incremental]" >&2
        exit 1
        ;;
esac
