---
name: deliver
description: Run the Tyrus autonomous delivery loop — pick the next WorkUnit from the roadmap/issues and drive it through every phase (plan, TDD, adversarial review, gates, PR, merge on green CI). Use to start or resume autonomous work.
---

# /deliver — autonomous delivery loop entry point

Playbook: `.claude/workflows/tyrus-feature-delivery.md` (read it; this skill is the
dispatcher, not the spec). Invariants: `.claude/rules/delivery.md`.

1. **Session grounding (P0):** `git checkout main && git pull`, `git status` clean,
   `cargo nextest run --workspace` baseline green. Verify the crate map claim you're about
   to rely on against `ls crates/`.
2. **Queue (P1):** if `$ARGUMENTS` names an issue/roadmap line, that is the WorkUnit.
   Otherwise build/refresh `.claude/plan/fila.md` from `gh issue list` + `ROADMAP.md`
   (order: CRITICAL bugs > Now > Next) and take the head. Assign the tier
   (trivial/medium/large) and write the unit's surface.
3. **Memory check (P2):** verify every auto-memory claim touching the surface against the
   real code before using it; rewrite what's wrong.
4. **Phases P3→P7** per the playbook, with the agent team it assigns per phase:
   plan (tyrus-planner for medium+; plan-reviewer adversarial pass for large) →
   `/tdd-codegen` RED→GREEN →
   test-audit → tyrus-reviewer on the diff → `/gates` + observed equivalence →
   `/pre-pr` → PR → CI monitor → **merge on green (verify the SHA)** → delete branch →
   update ledger.
5. **Loop:** take the next unit. Re-run P0/P2 only if the session or surface changed.
6. **PARA conditions (stop and report, never force):** gate failed 2× · ADR-shaped
   ambiguity · release/versioning/history decisions · red-line risk · scope leak ·
   memory-vs-code conflict the code doesn't settle · queue empty.

Every handoff and gate records observed evidence (exit codes, pasted outputs) in the
unit's plan file. State survives compaction via `.claude/plan/` (F10) — update it before
any long-running step.

$ARGUMENTS: optional WorkUnit selector (issue number like `#188`, or a roadmap line).
