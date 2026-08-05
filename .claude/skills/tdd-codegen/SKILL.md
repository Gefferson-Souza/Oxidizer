---
name: tdd-codegen
description: RED→GREEN workflow for Tyrus codegen changes (F2/R5). Use when adding or fixing any TS construct translation — writes the semantic equivalence test first, then implements.
---

# /tdd-codegen — equivalence-test-first codegen work

1. **RED:** write the semantic equivalence test in `tests/src/equivalence/<area>.rs`
   using `assert_output_equivalent(ts_code)` (or `_with_timeout` for state/mutex
   paths). The TS snippet must be valid TypeScript. Run it and PASTE the observed
   failure — if it doesn't fail, the test is wrong or the feature already works.
2. **GREEN:** implement in `crates/tyrus_codegen` under the codegen rules (quote!-only,
   registry for name-keyed logic, no panics, ≤ 50-line functions). Dispatch
   `tyrus-codegen` for the implementation when delegating.
3. Run the test until green, then the full suite (`cargo nextest run --workspace`)
   and `cargo clippy --workspace --all-targets --locked`.
4. If snapshots changed: `cargo insta review`, accept intentional diffs, commit the
   `.snap` in the same change.
5. Done = observed equivalence (F6), never "the tokens look right".

$ARGUMENTS: the TS construct to support/fix (e.g. "Array.prototype.flat", "getters em interfaces").
