---
paths:
  - "tests/**"
  - "crates/tyrus_test_utils/**"
---

# Testing Rules (binding — R5, F2, F6)

- **Layer selection** (`tests/src/`): `unit/` asserts on generated Rust text (fast);
  `snapshot/` uses insta on fixtures; `compilation/` runs real `cargo check`;
  `equivalence/` is the load-bearing layer — run the TS in Node 22 and the compiled
  Rust, compare stdout byte-for-byte. Behavior changes REQUIRE an equivalence test
  written RED first (F2); "the code looks right" is never done (F6).
- Helpers: `transpile()`, `assert_output_equivalent()` (+ `_with_timeout` for
  deadlock-prone paths — prefer it for mutex/state tests), `transpile_fixture()`.
  Test crate name is `integration_tests` (`cargo test -p integration_tests`).
- Fixtures live in `tests/fixtures/tier1..tier4` + `invalid/` + `uat/`. Fixture TS
  must be VALID TypeScript (tsc-clean) unless it exists to exercise the analyzer.
- Ports: never 3000–3002 (dev machines); the HTTP equivalence test owns 3100.
- insta: never commit `.snap.new`; run `cargo insta review` and commit the accepted
  snapshot in the same PR as the codegen change.
- Test code MAY use `.expect()`/panicking indexing — the crate roots already carry
  the scoped `#![allow]`s; do not widen them.
- Shared compile cache: generated-code tests build into `$TMPDIR/tyrus_test_target`.
  If disk pressure strikes, delete that dir (rebuildable) — never `~/.cargo`.
