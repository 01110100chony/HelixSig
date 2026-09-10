#!/usr/bin/env bash
set -euo pipefail
[[ $# -ge 2 ]] || { echo "Usage: $0 <profile-name> <task>" >&2; exit 2; }
PROFILE="$1"; shift; TASK="$*"
PROFILE_FILE="agent-prompts/antigravity/${PROFILE}.md"
[[ -f "$PROFILE_FILE" ]] || { echo "Unknown profile: $PROFILE_FILE" >&2; exit 2; }
if command -v agy >/dev/null 2>&1; then
  AGY_BIN="$(command -v agy)"
elif command -v agy.exe >/dev/null 2>&1; then
  AGY_BIN="$(command -v agy.exe)"
else
  echo "agy CLI not found (checked agy and agy.exe)" >&2
  exit 127
fi
mkdir -p .agent-runs
TS="$(date +%Y%m%d-%H%M%S)"; OUT=".agent-runs/${TS}-agy-${PROFILE}.md"
if [[ "${AGY_READ_ONLY:-1}" == "1" ]]; then
  MODE="read-only"
  GUARDRAIL="STRICTLY READ-ONLY. Do not edit, create, delete, rename, move, format, or generate files. Do not install dependencies or run commands that mutate git, the working tree, or external state. Use tools only to inspect and reason. If the task conflicts with these restrictions, report the conflict and stop."
else
  [[ -n "${AGY_WRITE_FILES:-}" ]] || { echo "Mutating task requires explicit AGY_WRITE_FILES (semicolon-separated absolute paths)" >&2; exit 2; }
  MODE="bounded-write"
  GUARDRAIL="BOUNDED MUTATING TASK. You may modify only these explicitly authorized absolute paths: ${AGY_WRITE_FILES}. Do not modify any other file or external state. Do not run git commit, reset, checkout, clean, restore, push, or other destructive commands. Stop and report if the task requires broader changes."
fi
MODEL="${AGY_MODEL:-gemini-3.8-flash-high}"
EFFORT="${AGY_EFFORT:-high}"
PROMPT="${GUARDRAIL}

$(cat "$PROFILE_FILE")

TASK:
${TASK}"
CMD=("$AGY_BIN" -p "$PROMPT" --model "$MODEL" --effort "$EFFORT" --dangerously-skip-permissions --output-format text)
{
  echo "# Antigravity subagent: ${PROFILE}"; echo; echo "- timestamp: ${TS}"; echo "- mode: ${MODE}"; echo "- model: ${MODEL}"; echo "- effort: ${EFFORT}"; echo "- task: ${TASK}"; echo; echo "## Response"; echo
  "${CMD[@]}"
} | tee "$OUT"
echo "Saved: $OUT" >&2
