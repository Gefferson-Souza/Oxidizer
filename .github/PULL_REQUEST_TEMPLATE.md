# 🏗️ Pull Request

## 📌 Context

**Issue Link:** # (or JIRA-123)
**Type:**

- [ ] ✨ New Feature (`feat`)
- [ ] 🐛 Bug Fix (`fix`)
- [ ] 🧹 Chore/Refactor (`chore`, `refactor`)
- [ ] 📚 Documentation (`docs`)

## 📝 Summary

<!-- Brief description of what was changed and why. -->

## 🧪 Test Plan

<!-- How did you verify this change? -->

- [ ] **Unit Tests**: `cargo nextest run --workspace`
- [ ] **Snapshot Tests**: `cargo test -p tests snapshot` (run `cargo insta review` if snapshots changed)
- [ ] **Compilation Tests**: `cargo test -p tests compilation`
- [ ] **Manual Verification**: (Describe steps)

## 📸 Screenshots / Logs

<!-- If applicable, add evidence here. -->

## ✅ Checklist

- [ ] I have performed a self-review of my code.
- [ ] I have added tests that prove my fix is effective or that my feature works.
- [ ] `cargo nextest run --workspace` passes locally.
- [ ] `cargo fmt -- --check` passes (no formatting issues).
- [ ] `cargo clippy --workspace -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::panic` passes with no warnings.
- [ ] No `.unwrap()`, `.expect()`, `panic!()`, `todo!()`, or `unimplemented!()` introduced.
- [ ] I have commented my code, particularly in hard-to-understand areas.
