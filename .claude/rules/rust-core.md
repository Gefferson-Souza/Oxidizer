---
paths:
  - "crates/**/*.rs"
---

# Rust Core Rules (binding — POWER_OF_TEN.md R4/R6/R13)

- **Never** emit `.unwrap()`, `.expect(`, `panic!`, `todo!`, `unimplemented!`, `dbg!`,
  `x[i]` indexing, or `&s[a..b]` slicing in production code. Use `Result<T, TyrusError>`,
  `?`, `.get()`, slice patterns (`if let [a, b] = xs`). Test modules may use
  `.expect("msg")` under `#[allow(clippy::expect_used)]`.
- Functions ≤ 50 lines, ≤ 5 params, nesting ≤ 4; files ≤ 400 lines (`gates.sh filesize`).
  Escape hatch: `#[expect(clippy::too_many_lines, reason = "…")]` only for coherent
  `quote!` templates — never `#[allow]`, always with a written reason.
- Every crate root carries `#![forbid(unsafe_code)]` (R13). Never remove it; new crates
  must add it (the `unsafe` gate fails otherwise).
- Lint policy lives ONLY in `[workspace.lints]` (root Cargo.toml). Never add rustflags
  to `.cargo/config.toml`, and never add crate-root lint attributes that duplicate the
  workspace table.
- Prefer `&self` over `&mut self`; `pub(crate)` over `pub` for internal APIs; newtypes
  over raw `String`/`PathBuf` for domain values (see `tyrus_common::fs::FilePath`).
- Comments state only the non-obvious *why*, in English, ≤ 4 lines. Never narrate what
  a line does, never reference the review/PR process.
- Before pushing: `./scripts/gates.sh all` green locally (F4). If run staged, every
  gate must pass on the exact pushed tree and the commit body must say so.
