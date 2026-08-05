---
name: doc-sync
description: Synchronize every Tyrus document that states counts, trees, commands or rules (F7). Use inside any PR that changes those facts, or standalone to repay doc drift.
---

# /doc-sync — make the documents match reality

1. Establish ground truth by MEASURING (never by copying from another doc):
   - test count: `cargo nextest run --workspace` summary line
   - crate/module trees: `ls crates/ crates/tyrus_codegen/src/**`
   - rule/gate lists: `docs/standards/POWER_OF_TEN.md` + `scripts/gates.sh` dispatcher
   - ADR list: `ls docs/architecture/decisions/`
2. Sync targets: `README.md`, `README.pt-br.md`, `docs/ARCHITECTURE.md` (incl. ADR
   index), `docs/specs/GRAMMAR.md`, `CONTRIBUTING.md`, `CLAUDE.md` quick table,
   `benches/README.md`. **Never `CHANGELOG.md`** (release-plz owns it).
3. Enforcement claims follow ADR 0013 honest-enforcement: active mechanisms name the
   gate/lint/test; pending ones name the tracking issue. No aspirational claims.
4. Known chronic drift points (check every run): test counts, "N crates/N rules"
   phrases, codegen module tree (decorators/, stdlib/, expr/call_array.rs,
   type_decl.rs, integer_heuristic.rs), lint-rule counts, benches README describing
   nonexistent benchmarks.
5. Dispatch `tyrus-docs` for the mechanical pass when delegating; review its old→new
   claim list against your measurements.
