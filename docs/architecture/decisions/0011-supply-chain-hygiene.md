# 11. Supply-Chain Hygiene Policy

Date: 2026-05-03
Status: Accepted (retroactive — backfill of PR #126)

## Context

PR #126 (`dfae3f2` — `chore(ci): hardening`) introduced four supply-chain controls in a single bundled commit:

1. `cargo deny` configuration (`deny.toml`, schema v2) — license allowlist + advisory ignore list + duplicate-version policy.
2. `cargo audit` integration in `scripts/gates.sh` — RustSec advisory database scan with explicit `--ignore` for triaged advisories.
3. `dependabot.yml` — weekly grouped updates for `cargo`, `github-actions`, `npm`.
4. `CODEOWNERS` — review approval routing.

Each control is a distinct policy choice. Bundling them violated Rule 11 (one branch = one concern) — a fact this ADR explicitly acknowledges. The retroactive ADR repays the documentation debt by re-stating each policy as a numbered decision with rationale, so future revisions can target individual policies without re-deriving the bundle's intent from PR diff archaeology.

## Decision

Tyrus enforces supply-chain hygiene through four numbered policies.

### Policy 1 — License allowlist

Allowed SPDX identifiers (`deny.toml [licenses].allow`):

```
MIT, Apache-2.0, Apache-2.0 WITH LLVM-exception,
BSD-2-Clause, BSD-3-Clause, ISC,
Unicode-3.0, Unicode-DFS-2016,
OpenSSL, Zlib, BSL-1.0, CC0-1.0, MPL-2.0
```

Rationale: covers the long-tail of permissive licenses present in the Rust ecosystem (notably `unicode-ident`, `ring`, `webpki-roots`) without admitting copyleft-with-derivatives risks. `confidence-threshold = 0.8` rejects ambiguous SPDX matches. New licenses require an ADR addendum, not a silent `deny.toml` edit.

### Policy 2 — Advisory ignore list

`cargo deny` and `cargo audit` ignore the following RustSec advisories pending follow-up:

| Advisory | Crate | Reason | Resolution |
|---|---|---|---|
| `RUSTSEC-2025-0119` | `number_prefix` (transitive via `indicatif`) | Unmaintained marker, not a vulnerability | Resolves once `indicatif` drops the dep |
| `RUSTSEC-2026-0097` | `rand` | Soundness gap only with custom `CryptoRng` + specific logger pattern we never construct | Resolves on `rand ≥ 0.9` upgrade |

The list is duplicated between `deny.toml` (`[advisories].ignore`) and `scripts/gates.sh` (`gate_audit` — `cargo audit --ignore ...`). The duplication is intentional: `cargo-audit` does not read `deny.toml`. **Both lists must stay in sync** — they are the project's two-place source of triaged advisories. Issue #128 tracks both lists for upgrade.

Adding an entry requires: (a) a comment explaining the analysis, (b) a tracking issue or this ADR, (c) a planned upgrade path (not "indefinitely ignored"). Zero-justification ignores are forbidden.

### Policy 3 — Dependabot grouping and cadence

`.github/dependabot.yml` runs three update streams:

- **`cargo`** — weekly (Monday), max 5 open PRs. Two groups:
  - `swc` — pattern `swc_*` (the SWC family version-locks, so a single grouped bump reviews better than per-crate noise).
  - `dev-dependencies` — `dependency-type: development` (test/bench-only updates do not affect runtime).
  - Everything else (production dependencies outside the SWC family) updates as individual PRs so semver-incompatible bumps are reviewed in isolation.
- **`github-actions`** — weekly (Monday), max 3 PRs, **no grouping** (the action ecosystem changes slowly enough that single-PR review is fine).
- **`npm`** — **monthly**, max 2 PRs, scoped to `tests/fixtures/reference_nestjs/` (the reference NestJS fixture, used for E2E equivalence). Slower cadence because the fixture pins a stable NestJS surface intentionally.

Rationale: ungrouped weekly Dependabot floods with one PR per dep, masking semver-incompatible bumps in noise. Targeted grouping (SWC family + dev-deps) cuts cargo review surface meaningfully without hiding production bumps. The npm monthly cadence keeps the test fixture tracking the broader ecosystem without thrashing it.

### Policy 4 — CODEOWNERS membership

`CODEOWNERS` (at the repo root) routes review approval per path. The current rule set:

- `*` → maintainer (catch-all).
- `/crates/tyrus_codegen/`, `/crates/tyrus_analyzer/`, `/crates/tyrus_decorator_kinds/`, `/crates/tyrus_di/` → maintainer (compiler-core extra scrutiny).
- `/docs/architecture/decisions/`, `/docs/specs/` → maintainer (public-contract documents).
- `/.github/`, `/.cargo/`, `/clippy.toml`, `/deny.toml`, `/release-plz.toml` → maintainer (anything that changes the gates).

Per-crate ownership inherits from `*` until a contributor demonstrates sustained ownership of a crate (≥ 3 reviewed PRs, ≥ 6 months active). At that point, CODEOWNERS gains an additional reviewer for that crate via separate ADR.

Rationale: as Tyrus accepts ML-agent and external contributions, gated review is the simplest enforcement of the strict rules. The compiler-core and gates-changing surfaces are explicitly carved out so they can never be approved by a less-scoped contributor accidentally.

> **Note.** `docs/standards/POWER_OF_TEN.md` is not currently routed via CODEOWNERS — it lives under `/docs/standards/` which has no rule. The `*` catch-all covers it. A follow-up could add an explicit `/docs/standards/` rule to mirror the `/docs/architecture/decisions/` carve-out, but that is a CODEOWNERS edit, not an ADR-level decision.

## Consequences

### Positive

- **Numbered, citable policies.** Each future advisory triage / license addition / membership change has a specific section to amend, not an opaque bundle.
- **Audit trail.** Future contributors see *why* `RUSTSEC-2026-0097` is ignored without re-reading PR diffs.
- **Drift prevention.** The "two-place advisory list" gotcha (deny.toml ↔ gates.sh) is documented; future contributors are unlikely to update one and forget the other.

### Negative

- **No structural enforcement of "lists in sync".** A discipline failure can still let `deny.toml` and `gates.sh` drift. A future advisory test (`gate_audit_drift_check`) would close this; tracked as follow-up.
- **Solo maintainer until contributor base grows.** CODEOWNERS effectively routes everything to one person today. Once a second crate-owner exists, CODEOWNERS is the substitution point.
- **Rule 11 violation acknowledged but not mended.** PR #126 was bundled; this ADR documents the bundling as a known historical exception. Future bundled-policy PRs are still forbidden — the precedent is documentation-only, not licence-to-bundle.

## Alternatives rejected

- **Wildcard license allow (`allow = ["*"]` or removing `deny.toml`).** Loses the audit trail. Allows GPL-like licenses without review. Rejected.
- **Per-crate audit-only (drop `cargo deny`).** `cargo audit` covers vulnerabilities but not licenses, sources, or duplicate versions. Halves the coverage. Rejected.
- **Disable Dependabot.** Trade automated-but-noisy patches for manual-only patches → security advisories sit unfixed for weeks. Rejected.
- **No CODEOWNERS, allow free-for-all merges.** Inconsistent with strict-rule enforcement; a single maintainer cannot police every PR by skim. CODEOWNERS is the cheapest gate. Rejected.

## References

- Originating PR: [#126](https://github.com/Gefferson-Souza/Tyrus/pull/126) (`dfae3f2`) — `chore(ci): hardening`.
- Configurations: `deny.toml`, `scripts/gates.sh` (`gate_deny`, `gate_audit`), `.github/dependabot.yml`, `CODEOWNERS`.
- Tracking: issue #128 (advisory upgrade plan).
- Related: Rule 11 (POWER_OF_TEN.md — bundling acknowledged); Rule 12 (warnings-clean / daily audited).
