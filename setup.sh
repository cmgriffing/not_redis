#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOOKS_DIR="$SCRIPT_DIR/.git/hooks"
GIT_HOOKS_DIR="$SCRIPT_DIR/git_hooks"

echo "Setting up git hooks..."

for hook_file in "$GIT_HOOKS_DIR"/*.sh; do
    [ -f "$hook_file" ] || continue

    filename=$(basename "$hook_file")
    hook_name="${filename%.sh}"

    dest="$HOOKS_DIR/$hook_name"

    if [ -f "$dest" ]; then
        if ! diff -q "$hook_file" "$dest" > /dev/null 2>&1; then
            echo "  Warning: Replacing existing hook: $hook_name"
            [ -f "$dest.backup" ] && rm -f "$dest.backup"
            mv "$dest" "$dest.backup"
            cp "$hook_file" "$dest"
            chmod +x "$dest"
            echo "  Backup saved: $hook_name.backup"
        fi
    else
        cp "$hook_file" "$dest"
        chmod +x "$dest"
        echo "  Installed: $filename -> $hook_name"
    fi
done

echo "Done!"
