# Delivery Invariants (always loaded — binding for autonomous work)

> Operational companion to `docs/standards/POWER_OF_TEN.md` (R1–R14) and
> `docs/standards/DEVELOPMENT_FLOW.md` (F1–F10). The full phase playbook is
> `.claude/workflows/tyrus-feature-delivery.md`; the loop entry point is `/deliver`.

1. **RULE ZERO — observed equivalence.** The project exists to produce Rust whose behavior
   is identical to the source TypeScript. Never claim a transpilation change works without
   having RUN both sides (Node vs compiled Rust) and compared outputs — stdout for programs,
   HTTP responses for the NestJS tier. "The tokens look right" and "snapshot updated" are
   not evidence (R5/F6).

2. **Never assume — validate before asserting.** Confirm claims against real code
   (`file:line`) and real command output before stating them. Memory, docs, and summaries
   are inputs to verify. When memory conflicts with code, code wins and the memory is
   rewritten. A query returning zero/empty needs a positive control before it proves absence.

3. **Focus.** The scope is the issue's stated scope — nothing else enters the branch.
   "While I'm here" changes become new issues. One branch = one concern (F1/R11).
   Work order: open CRITICAL bugs > ROADMAP.md **Now** > **Next**; the loop never invents
   work, and re-prioritizing the roadmap is the owner's call.

4. **Tests are gates, not decoration.** RED before GREEN for every behavior change (F2);
   a test that cannot fail when the key line is broken is vacuous — rewrite it
   (mutation-check). Fix the implementation, not the test, unless the test is proven wrong.

5. **Fresh-context review before every PR** (F5): the reviewer did not author the diff and
   is never told what not to flag. CRITICAL/HIGH block; sustained findings get fixed, not
   argued away.

6. **Delegation restates constraints** (F8): every subagent prompt carries the binding
   F/R rules, branch, commit format, paths not to touch, and output language — verbatim.
   Never assume a subagent read CLAUDE.md.

7. **Autonomy boundary.** The loop may run issue → branch → TDD → review → PR → merge with
   green CI on its own. It STOPS for the owner on: a gate failing 2×, ADR-shaped ambiguity,
   release/versioning/git-history decisions, red-line risk, scope leaks, or roadmap
   re-prioritization. Never force-push, never edit CHANGELOG.md, never merge on a CI green
   that belongs to a different SHA.
