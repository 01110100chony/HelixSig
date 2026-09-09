#!/usr/bin/env bash
set -euo pipefail
TASK="${1:-Review the current milestone against its active plan.}"
scripts/agents/copilot-agent.sh cpp-reviewer "$TASK" & P1=$!
scripts/agents/copilot-agent.sh rust-concurrency-reviewer "$TASK" & P2=$!
scripts/agents/copilot-agent.sh test-adversary "$TASK" & P3=$!
wait "$P1" "$P2" "$P3"
echo "Parallel reviews complete. Codex must adjudicate .agent-runs/."
