# 14. Development Flow Rules (F1–F10)

Date: 2026-08-04
Status: Accepted

## Context

ADR 0008 adopted the Tyrus Power of Ten — binding rules for *code*. Nothing equivalent governed the *process* that produces the code. The gap showed up as concrete, recurring failures:

- **Doc drift:** README, ARCHITECTURE.md and CHANGELOG.md reached three contradictory test counts (195/230/235); the README module tree was missing six files that exist on disk; the ADR index stopped at 0011 while 0012 existed.
- **Stalled campaigns:** the 2026-05-07 UAT bug-fix campaign merged three of eight planned PRs and stopped; the merged branch remained the checked-out HEAD for months; four merged branches accumulated locally and remotely.
- **Hidden CI rot:** the Benchmarks job was red for weeks (a raw-token `fn main` detection bug), which masked a second, unrelated breakage (clippy 1.97's new `question_mark` hits) — every open PR showed red and none of the failures was triaged.
- **Delegation loss:** work delegated to AI subagents silently dropped project constraints (commit format, forbidden paths) that were never restated in the delegation prompt, because subagents start from fresh context by design.
- **Stale state:** project memory described a seven-agent team whose definition files no longer existed.

This project is developed primarily through AI-agent-driven flows (single agents, subagent orchestration, multi-PR pipelines), which amplifies process failures: an unstated convention simply does not exist for a fresh-context agent.

## Decision

Adopt `docs/standards/DEVELOPMENT_FLOW.md` as a second normative standard, peer to POWER_OF_TEN.md, with ten numbered rules:

- **F1 — Issue-First, One Branch = One Concern** (extends R11)
- **F2 — Test-First for Behavior** (extends R5)
- **F3 — Plan-First for Non-Trivial Work** (extends R10)
- **F4 — Local Gates Green Before Push** (extends R9)
- **F5 — Two-Layer Review** (fresh-context review + adversarial verification)
- **F6 — Observed Evidence Defines "Done"**
- **F7 — Docs Ship in the Same PR** (release-plz owns CHANGELOG.md)
- **F8 — Delegation Repeats Constraints** (AI subagent prompts restate binding rules)
- **F9 — ADR Before the PR** (extends R10; index updated in the same commit)
- **F10 — State Lives in Its System of Record** (plans vs memory vs issues)

Structural choices, mirroring ADR 0008:

1. **Two documents, not one.** Code rules (R) and flow rules (F) have different audiences and different enforcement surfaces (clippy/CI vs templates/agents/checklists). Merging them was rejected: a 600-line super-document would defeat the "read the whole standard in five minutes" property that gives POWER_OF_TEN.md its authority.
2. **One rule = statement + why + enforcement + severity.** A rule with no enforcement mechanism is not accepted into the document.
3. **Explicit composition table.** Every F-rule declares which R-rule it extends, guaranteeing zero overlap — F-rules add process obligations, never restate code obligations.

## Consequences

**Positive**

- Process failures become review-blocking findings with a rule number, instead of unactionable "be more careful" feedback.
- AI delegation gains a contract (F8): agent definitions under version control must carry the constraints they operate under.
- Documentation debt stops accumulating by construction (F7) rather than being repaid in periodic sync campaigns.
- The staged-gates escape hatch (F4) legitimizes running gates in stages when the monolithic hook run is impractical, while keeping the all-gates-green-before-push invariant.

**Negative / accepted costs**

- More ceremony per change: an issue and (for behavior changes) a failing test precede any implementation. Accepted: this is the cost that keeps multi-agent pipelines auditable.
- PR template grows additional checkboxes (F1, F2, F7, F9); template update ships separately.
- Rules F3, F5, F8 and F10 are enforced by discipline and review rather than deterministic tooling; follow-up work (versioned agent definitions, hooks) will harden them.

## Alternatives rejected

- **Single unified STANDARDS.md** — rejected (see Decision, point 1).
- **One file per rule** (Safety-Critical Rust Consortium layout) — rejected: 24 files for a single-maintainer project loses whole-standard readability; the Consortium mapping is instead an annex planned for POWER_OF_TEN.md.
- **Encode everything as hooks/CI now** — rejected as scope for this ADR: enforcement automation lands with the Claude Code harness work; the standard must exist first so the harness has something to enforce.

## References

- [DEVELOPMENT_FLOW.md](../../standards/DEVELOPMENT_FLOW.md) — the standard this ADR adopts.
- [POWER_OF_TEN.md](../../standards/POWER_OF_TEN.md) / [ADR 0008](0008-tyrus-strict-rules.md) — the code-rule companion and its adoption precedent.
- NASA/JPL "The Power of Ten — Rules for Developing Safety-Critical Code" (Holzmann, 2006) — severity/enforcement structure mirrored here.
