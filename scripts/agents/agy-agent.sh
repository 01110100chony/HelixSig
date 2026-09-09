#!/usr/bin/env bash
set -euo pipefail
[[ $# -ge 2 ]] || { echo "Usage: $0 <profile-name> <task>" >&2; exit 2; }
PROFILE="$1"; shift; TASK="$*"
PROFILE_FILE="agent-prompts/antigravity/${PROFILE}.md"
[[ -f "$PROFILE_FILE" ]] || { echo "Unknown profile: $PROFILE_FILE" >&2; exit 2; }
mkdir -p .agent-runs
TS="$(date +%Y%m%d-%H%M%S)"; OUT=".agent-runs/${TS}-agy-${PROFILE}.md"
PROMPT="$(cat "$PROFILE_FILE")

TASK:
${TASK}"
CMD=(agy -p "$PROMPT" --output-format text)
[[ -z "${AGY_MODEL:-}" ]] || CMD+=(--model "$AGY_MODEL")
[[ -z "${AGY_EFFORT:-}" ]] || CMD+=(--effort "$AGY_EFFORT")
{
  echo "# Antigravity subagent: ${PROFILE}"; echo; echo "- timestamp: ${TS}"; echo "- task: ${TASK}"; echo; echo "## Response"; echo
  "${CMD[@]}"
} | tee "$OUT"
echo "Saved: $OUT" >&2
