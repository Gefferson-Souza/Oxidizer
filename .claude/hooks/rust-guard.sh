#!/bin/bash
# PreToolUse guard (Edit/Write): blocks Rule 6/7 violations at edit time,
# before clippy ever runs. Deterministic layer of POWER_OF_TEN.md — the
# gates remain the authoritative check; this only fails fast on the
# obvious cases. Exit 2 blocks the tool call; anything else lets it pass.

set -u
payload=$(cat)

command -v jq >/dev/null 2>&1 || exit 0

file_path=$(jq -r '.tool_input.file_path // empty' <<<"$payload")
new_text=$(jq -r '.tool_input.content // .tool_input.new_string // empty' <<<"$payload")

[ -z "$file_path" ] && exit 0
[ -z "$new_text" ] && exit 0

case "$file_path" in
  *crates/*/src/*.rs) ;;
  *) exit 0 ;;
esac

# Test modules legitimately use expect/panicking asserts (Rule 6 carve-out).
if grep -qE '#\[cfg\(test\)\]|#\[test\]' <<<"$new_text"; then
  exit 0
fi

# Rule 6 — forbidden panic constructs in production code.
banned=$(grep -nE '\.unwrap\(\)|\.expect\(|panic!\(|todo!\(|unimplemented!\(|dbg!\(' <<<"$new_text" || true)
if [ -n "$banned" ]; then
  echo "BLOCKED (Rule 6, POWER_OF_TEN.md): panic construct in production code:" >&2
  echo "$banned" | head -5 >&2
  echo "Use Result<T, TyrusError>, ?, .get(), or compile_error! in generated code." >&2
  echo "Test code goes in #[cfg(test)] modules with #[allow(clippy::expect_used)]." >&2
  exit 2
fi

# Rule 7 — string-assembled Rust source inside codegen conversion modules.
case "$file_path" in
  *crates/tyrus_codegen/src/convert/*)
    if grep -qE 'format!\("(fn |pub |impl |struct |use )' <<<"$new_text"; then
      echo "BLOCKED (Rule 7, POWER_OF_TEN.md): string-concatenated Rust in convert/." >&2
      echo "Emit tokens with quote! — never format! of Rust source." >&2
      exit 2
    fi
    ;;
esac

exit 0
