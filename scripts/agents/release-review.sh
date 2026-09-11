#!/usr/bin/env bash
set -euo pipefail
TASK="${1:-Perform an independent closeout review of the current release candidate.}"
scripts/agents/copilot-agent.sh cpp-reviewer "$TASK" & P1=$!
scripts/agents/copilot-agent.sh rust-concurrency-reviewer "$TASK" & P2=$!
scripts/agents/copilot-agent.sh benchmark-auditor "$TASK" & P3=$!
scripts/agents/agy-agent.sh independent-reviewer "$TASK" & P4=$!
status=0
for pid in "$P1" "$P2" "$P3" "$P4"; do
  wait "$pid" || status=1
done
if [[ "$status" != 0 ]]; then
  echo "One or more reviewers failed; inspect .agent-runs/." >&2
  exit "$status"
fi
echo "Release reviews complete. Codex must adjudicate before PASS/FAIL."
