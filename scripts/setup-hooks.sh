#!/bin/sh
# Setup git hooks for Tyrus development
# Usage: ./scripts/setup-hooks.sh

HOOKS_DIR="$(git rev-parse --show-toplevel)/.git/hooks"
SCRIPTS_DIR="$(git rev-parse --show-toplevel)/scripts"

ln -sf "$SCRIPTS_DIR/pre-commit" "$HOOKS_DIR/pre-commit"
echo "Installed pre-commit hook"
echo "Done. Hooks are active."
