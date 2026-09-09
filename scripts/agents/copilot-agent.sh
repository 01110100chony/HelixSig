#!/usr/bin/env bash
set -euo pipefail
[[ $# -ge 2 ]] || { echo "Usage: $0 <agent-name> <task>" >&2; exit 2; }
AGENT="$1"; shift; TASK="$*"
mkdir -p .agent-runs
TS="$(date +%Y%m%d-%H%M%S)"; OUT=".agent-runs/${TS}-copilot-${AGENT}.md"
CMD=(copilot "--agent=${AGENT}" "--prompt=${TASK}")
[[ -z "${COPILOT_MODEL:-}" ]] || CMD+=("--model=${COPILOT_MODEL}")
{
  echo "# Copilot subagent: ${AGENT}"; echo; echo "- timestamp: ${TS}"; echo "- task: ${TASK}"; echo; echo "## Response"; echo
  "${CMD[@]}"
} | tee "$OUT"
echo "Saved: $OUT" >&2
