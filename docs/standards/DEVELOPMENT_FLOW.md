# Tyrus Development Flow Rules

> **Strict rules for the Tyrus development process.** Companion to [POWER_OF_TEN.md](./POWER_OF_TEN.md): that document governs the *code*; this one governs the *flow* that produces it — issues, branches, tests-first, review, documentation, and delegation to AI agents.
>
> **Status:** Accepted (ADR 0014 / 2026-08-04). Binding for all contributions to `main`.

---

## Why these rules

The Power of Ten closes code-level failure modes. This project has also lived through *process*-level failure modes, each of which one of these rules closes:

1. **Doc drift** — README, ARCHITECTURE and CHANGELOG disagreeing on test counts and module trees because docs were updated "later" (i.e., never).
2. **Stalled campaigns** — multi-PR effort left half-merged, with the merged branch still checked out as HEAD and stale local branches accumulating.
3. **Silent scope creep** — fixes that grow into refactors mid-PR, breaking `git bisect` and review focus.
4. **Delegation loss** — AI subagents starting from fresh context and silently ignoring project constraints that were never repeated in their prompt.
5. **"Looks done" completions** — visual/integration work declared finished from code inspection instead of observed output.

Rules are numbered F1–F10 to compose with the code rules (R1–R14). Each F-rule states which R-rule it extends, if any. **A rule with no enforcement is just a wish** — every rule names its enforcement mechanism.

---

## The 10 rules

### Rule F1 — Issue-First, One Branch = One Concern

**Rule.** No production change without a GitHub issue describing the problem *before* the branch exists. Branch names follow `<type>/<slug>` and reference the issue in the first commit or PR body. One branch carries exactly one logical concern (extends R11). Bot-authored tracking artifacts (e.g., dependabot PRs) satisfy the issue requirement for their own scope.

**Why.** Issues are the audit trail that survives squash merges and history rewrites. Concern-mixing is how "fix clippy lint" quietly becomes "refactor the analyzer" and breaks bisectability.

**Enforce.** PR template checkbox ("Closes #N"); review rejects PRs whose diff exceeds the issue's stated scope.

**Severity.** HIGH (review-blocking).

---

### Rule F2 — Test-First for Behavior

**Rule.** Any change to codegen or analyzer behavior starts with a failing test (RED) — a semantic equivalence test for codegen (extends R5), a reproduction test for bug fixes — before implementation (GREEN). Pure refactors with zero behavior change are exempt but must state so in the PR body.

**Why.** A test written after the fix proves nothing about the fix; a test written before it proves the bug existed and is now closed. Equivalence density (R5) is only meaningful if the test predates the change it guards.

**Enforce.** PR template requires linking the test added/modified; reviewers check the test fails on `main` and passes on the branch.

**Severity.** CRITICAL (merge-blocking for codegen/analyzer changes).

---

### Rule F3 — Plan-First for Non-Trivial Work

**Rule.** Work touching ≥ 2 crates, introducing a new public trait, or adding a new concept requires a written plan (`.claude/plan/` or issue body) reviewed *before* implementation starts. Architectural decisions additionally require an ADR before the PR (extends R10, see F9).

**Why.** Plans reviewed after the code exists are rubber stamps. The cheapest place to catch a wrong design is before it is typed.

**Enforce.** Process discipline + adversarial plan review (fresh-context reviewer agent) for multi-crate work.

**Severity.** HIGH.

---

### Rule F4 — Local Gates Green Before Push

**Rule.** `scripts/gates.sh all` passes locally before every push (extends R9). CI is a *verifier*, never a *debugger*: a failure reproducible on the dev machine must be reproduced and fixed there. When the monolithic hook run is impractical (resource limits), gates may be run staged — but **all** gates must pass on the exact tree being pushed, and the commit message must say so.

**Why.** Using CI as a debugger burns 10-minute round-trips on failures a local run catches in seconds, and normalizes red CI — which then masks *new* failures (this repo's Benchmarks job was red for weeks and nobody noticed the second, unrelated breakage it hid).

**Enforce.** Pre-commit hook (`scripts/pre-commit` → `gates.sh all`); `--no-verify` is permitted only with a staged full-gates run documented in the commit body.

**Severity.** CRITICAL.

---

### Rule F5 — Two-Layer Review

**Rule.** Every PR passes (a) author self-review against the PR-template checklist, and (b) an independent review in *fresh context* — a reviewer (human or agent) that did not participate in writing the diff. Findings rated CRITICAL or HIGH block merge; MEDIUM findings become issues.

**Why.** The author's context is contaminated by intent — they read what they meant, not what they wrote. Fresh-context review is the only reliable detector of "plausible but wrong" output, which is the signature failure mode of AI-generated code.

**Enforce.** PR template review section; for codegen/analyzer diffs, findings are adversarially verified (a second agent attempts to refute each finding) before they block or pass.

**Severity.** CRITICAL.

---

### Rule F6 — Observed Evidence Defines "Done"

**Rule.** A change is done when its effect has been *observed*, not inferred: codegen changes show the generated Rust compiling and producing equivalent output (extends R5); CLI changes show real exit codes and stdout; HTTP features show real responses. "The code looks correct" is never an acceptance statement.

**Why.** Every UAT-critical bug this project logged (top-level statements dropped, `check` exiting 0 on errors, `build` skipping the analyzer) shipped behind code that *looked* correct. Only observed behavior catches the gap between intention and semantics.

**Enforce.** Equivalence/E2E test layers; PR template "Test plan" section requires pasted observed output for behavioral changes.

**Severity.** CRITICAL.

---

### Rule F7 — Docs Ship in the Same PR

**Rule.** A PR that changes behavior, module structure, commands, or counts updates every document that states them (README, ARCHITECTURE, GRAMMAR, this file) *in the same PR*. CHANGELOG.md is never edited by hand — release-plz owns it. Documentation debt is not carried as "follow-up".

**Why.** Doc drift is cumulative and quadratic to repay: this repo reached three contradictory test counts and a module tree missing six files. Same-PR docs cost minutes; a doc-sync campaign costs days.

**Enforce.** PR template checkbox; reviewer checks affected-doc list against the diff.

**Severity.** HIGH.

---

### Rule F8 — Delegation Repeats Constraints

**Rule.** Every prompt that delegates work to an AI subagent restates, verbatim, the constraints that bind that task: the applicable F/R rules, target branch, commit format, paths it must not touch, output format, and language. No delegation may assume the subagent inherits CLAUDE.md, memory, or session context.

**Why.** Subagents start from fresh context by design; several agent types skip project config *deliberately* for speed. An unstated constraint is, for the subagent, a nonexistent constraint — this is a silent-failure mode with no error message.

**Enforce.** Agent definitions under version control carry constraint blocks; orchestrator checklists; review of agent output against the constraints it was given.

**Severity.** HIGH.

---

### Rule F9 — ADR Before the PR

**Rule.** Decisions matching R10's trigger criteria — plus two flow-level triggers R10 does not name: adopting a new normative rule, and adopting or rejecting tooling — are recorded as an ADR under R10's terms. This rule adds one obligation: the ADR index in `docs/ARCHITECTURE.md` is updated in the same commit that adds the ADR.

**Why.** An ADR written after the fact documents what happened; an ADR written before documents why it should. Rejected alternatives (with reasons) are the most valuable part and are only honest when written pre-implementation.

**Enforce.** PR template ADR checkbox; reviewers reject architecture-shaped diffs with no ADR reference.

**Severity.** HIGH.

---

### Rule F10 — State Lives in Its System of Record

**Rule.** Operational state (campaign progress, unit ledgers, next steps) lives in `.claude/plan/` files updated at the end of every working session. Durable learnings (corrections, non-obvious decisions, external API quirks) live in agent memory. GitHub issues/PRs are the record for work items. No system holds another's data: memory never tracks PR status; plans never hold learnings; issues never carry session state.

**Why.** When state is written to the wrong store it expires silently: memory said this project had a seven-agent team that no longer existed; a merged branch stayed checked out as HEAD because session state lived nowhere. Recovery from a correct ledger takes minutes; from stale memory, hours of re-discovery.

**Enforce.** Session-end checklist (update active plan, mark ledger); periodic memory audit against reality.

**Severity.** MEDIUM (warning + cleanup issue).

---

## Composition with the Power of Ten

| F-rule | Extends | Adds |
|--------|---------|------|
| F1 | R11 | issue-first requirement, bot-artifact exemption |
| F2 | R5 | RED-before-GREEN ordering, refactor exemption |
| F3 | R10 | plan review gate before implementation |
| F4 | R9 | staged-gates escape hatch with documentation duty |
| F5 | — | fresh-context review + adversarial verification |
| F6 | R5 | observed-output acceptance criterion |
| F7 | — | same-PR documentation duty, release-plz ownership |
| F8 | — | constraint restatement for AI delegation |
| F9 | R10 | two flow-level triggers (normative rules, tooling), same-commit index update |
| F10 | — | system-of-record separation |

## Severity legend

Severities carry the same meaning as in [POWER_OF_TEN.md](./POWER_OF_TEN.md): **CRITICAL** blocks merge, **HIGH** blocks review approval, **MEDIUM** is a warning plus follow-up issue.

## Revision

Changes to this document follow F9: an ADR superseding ADR 0014, then the PR.
