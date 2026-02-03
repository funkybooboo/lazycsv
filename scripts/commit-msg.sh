#!/bin/sh

COMMIT_MSG_FILE=$1
COMMIT_MSG=$(head -n1 "$COMMIT_MSG_FILE")

# Regex for Conventional Commits
CONVENTIONAL_COMMIT_REGEX="^(feat|fix|docs|style|refactor|test|chore)(\(.+\))?: .+"

if ! echo "$COMMIT_MSG" | grep -qE "$CONVENTIONAL_COMMIT_REGEX"; then
    echo "Error: Your commit message does not follow the Conventional Commits format." >&2
    echo "Please format your commit message as: <type>(<scope>): <description>" >&2
    echo "Example: feat(api): add new endpoint" >&2
    exit 1
fi
