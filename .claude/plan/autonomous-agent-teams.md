# Autonomous Agent Teams — Tyrus Development Pipeline

## Task Type
- [x] Backend (Rust transpiler development)

## Technical Solution

Use Claude Code's official **Agent Teams** feature (experimental, already enabled) to coordinate parallel development of the Tyrus transpiler. The Team Lead (main session) orchestrates specialist teammates through a shared task list.

### Agent Teams Setup

Already enabled in `~/.claude/settings.json`:
```json
{
  "env": { "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1" },
  "preferences": { "tmuxSplitPanes": true }
}
```

### Team Template — Per Feature

When starting a new feature from the roadmap, create a team with:

```
Create an agent team to implement [FEATURE NAME] from the NestJS roadmap.
Spawn 3 teammates:

1. "architect" — Read crates/tyrus_codegen/src/convert/ and analyze how the
   SWC AST represents [FEATURE]. Design the TS→Rust mapping. Report the design
   plan with file:line references. Require plan approval before proceeding.

2. "tester" — Write failing equivalence tests for [FEATURE] in
   tests/src/equivalence/[name].rs. Use assert_output_equivalent(). Wrap
   console.log in function main(): void { ... } main(); pattern.
   Register module in tests/src/equivalence/mod.rs.

3. "reviewer" — After implementation is complete, review ALL changed files
   for: no unwrap/expect/panic, files<400 lines, functions<50 lines,
   pub(crate) not pub, quote! macros only. Report issues by severity.

Use Sonnet for teammates. Require plan approval for the architect.
```

### Implementation Steps

1. **Step 0 — Context Load**: Read memory, roadmap, verify baseline tests
2. **Step 1 — Issue + Branch**: `gh issue create` → `git checkout -b`
3. **Step 2 — Spawn Team**: Create agent team with architect + tester + reviewer
4. **Step 3 — Parallel Research**: Architect designs, tester writes RED tests
5. **Step 4 — Lead Reviews Architect Plan**: Approve or reject with feedback
6. **Step 5 — Implement**: Lead or codegen teammate makes tests GREEN
7. **Step 6 — Quality Gates**: `cargo fmt` + `clippy` + `nextest`
8. **Step 7 — Reviewer Teammate Audits**: Fix CRITICAL/HIGH, re-review if needed
9. **Step 8 — Ship**: `git commit` + `git push` + `gh pr create`
10. **Step 9 — Docs**: Separate branch, separate PR
11. **Step 10 — Clean Up Team**: Shut down teammates, lead cleans resources
12. **Step 11 — Update Memory**: Test count, phase status
13. **Loop**: Back to Step 0 for next roadmap task

### Roadmap Progress (Phase 6)

| Milestone | Status | Tasks |
|-----------|--------|-------|
| 6.0 Infrastructure | ✅ COMPLETE | prettyplease, thiserror 2.0, trybuild |
| 6.1 try-catch | ✅ COMPLETE | analyzer unblock, closure-match, nested |
| 6.2 Top-level stmts | ✅ COMPLETE | has_declared_main, fn main() wrapper |
| 6.3 Spread/Rest | ✅ COMPLETE | array spread, rest params (declaration) |
| 6.4 Inheritance | ✅ COMPLETE | extends, super(), field flattening |
| 6.5 Static methods | ✅ COMPLETE | is_static, call-site ClassName::method() |
| **6.6 Type assertions** | **NEXT** | as Type → no-op or .to_string() |
| 6.6 Enums | NEXT | numeric enums, string enums enhanced |

### Key Files

| File | Purpose |
|------|---------|
| `~/.claude/settings.json` | Agent Teams enabled |
| `memory/workflow_autonomous_pipeline.md` | Persistent workflow memory |
| `memory/MEMORY.md` | Project state index |
| `~/.claude/agents/tyrus-*.md` | Agent role reference files (6 roles) |
| `docs/superpowers/plans/2026-03-13-nestjs-*.md` | Master roadmap |

### Risks and Mitigation

| Risk | Mitigation |
|------|------------|
| Agent Team token cost | Use subagents for simple 1-2 file changes |
| Teammates editing same file | Break work so each teammate owns different files |
| Team not cleaning up | Always tell lead to clean up before ending session |
| Teammate gets stuck | Lead sends guidance message directly |
| Session resumption loses teammates | Spawn fresh team on new session |
