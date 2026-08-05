#!/bin/bash
# PreToolUse guard (Bash): validates `git commit` messages against the
# repo convention (`<type>: description`, Conventional Commits, R11)
# before the command runs. Exit 2 blocks; anything else passes.

set -u
payload=$(cat)

command -v jq >/dev/null 2>&1 || exit 0

cmd=$(jq -r '.tool_input.command // empty' <<<"$payload")
[ -z "$cmd" ] && exit 0

case "$cmd" in
  *"git commit"*) ;;
  *) exit 0 ;;
esac

# Extract the first -m argument (subject line). Amend/no-message forms pass.
subject=$(printf '%s' "$cmd" | grep -oP -- '-m\s+"([^"\n]+)' | head -1 | sed 's/-m\s*"//')
[ -z "$subject" ] && exit 0

if ! grep -qE '^(feat|fix|refactor|docs|test|chore|perf|ci|style|build|revert)(\([a-z0-9._#/-]+\))?!?: .{8,}' <<<"$subject"; then
  echo "BLOCKED (R11/F1): commit subject must be '<type>: description' (Conventional Commits)." >&2
  echo "Got: $subject" >&2
  exit 2
fi

if grep -qiE '^(feat|fix|refactor|docs|test|chore|perf|ci|style|build|revert)(\([^)]*\))?!?: *(fix|ajustes?|updates?|wip|stuff|misc|changes?)\.?$' <<<"$subject"; then
  echo "BLOCKED (R11/F1): vague commit subject. Say WHAT changed and WHY." >&2
  echo "Got: $subject" >&2
  exit 2
fi

exit 0
