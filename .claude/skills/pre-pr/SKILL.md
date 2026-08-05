---
name: pre-pr
description: Full F5/F7 pre-PR checklist for Tyrus — gates, fresh-context adversarial review, doc sync, and PR body requirements. Use when an implementation slice is ready to become a PR.
---

# /pre-pr — everything that must be true before opening a PR

1. **F1:** an issue exists and the branch is `<type>/<slug>`; the PR will `Closes #N`.
2. **F4:** run `/gates` (all nine green on this exact tree). If staged, the commit
   body says so.
3. **F5:** dispatch `tyrus-reviewer` (fresh context, read-only) on the diff
   (`git diff main...HEAD` written to a scratch file — never pasted inline).
   CRITICAL/HIGH findings: fix and re-review. MEDIUM: fix or open an issue. Never
   tell the reviewer what not to flag.
4. **F2/F6:** for behavior changes, the PR body links the test that was RED first
   and pastes observed output (equivalence stdout, CLI exit codes, HTTP responses).
5. **F7:** if the change altered counts/trees/commands/rules, run `/doc-sync` — docs
   ship in this same PR. CHANGELOG.md untouched (release-plz).
6. **F9:** architecture-shaped change (R10 triggers) → the ADR merged before or
   within this PR, and the ADR index updated in the same commit.
7. PR body sections: Summary (what + why), `Closes #N`, Test plan with observed
   results. Conventional Commit title.
8. After opening: watch CI with a poll loop on `gh pr checks <n>` (this repo's `gh`
   has no `--json` for checks — parse the tab-separated table).
