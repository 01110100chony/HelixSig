#!/usr/bin/env bash
set -euo pipefail
[[ $# -eq 2 ]] || { echo "Usage: $0 <branch-name> <worktree-path>" >&2; exit 2; }
git worktree add "$2" -b "$1"
echo "Created worktree $2 on branch $1"
