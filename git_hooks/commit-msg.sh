#!/bin/bash
# Commit-msg hook: Validate commit message format

set -e

MSG_FILE="$1"

# Check that commit message follows conventional commits format
# Format: <type>(<scope>): <subject>
# Examples: "feat: add new command", "fix(cli): handle edge case"

echo "Validating commit message..."

if ! grep -qE '^[a-z]+(\([a-z0-9-]+\))?: .+' "$MSG_FILE"; then
    echo "Error: Commit message should follow conventional commits format:"
    echo "  <type>(<scope>): <subject>"
    echo ""
    echo "Examples:"
    echo "  feat: add new command"
    echo "  fix(cli): handle edge case"
    echo "  docs: update README"
    exit 1
fi

echo "Commit message validated!"
