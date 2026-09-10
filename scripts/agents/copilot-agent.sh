#!/usr/bin/env bash
set -euo pipefail
[[ $# -ge 2 ]] || { echo "Usage: $0 <agent-name> <task>" >&2; exit 2; }
AGENT="$1"; shift; TASK="$*"
command -v copilot >/dev/null 2>&1 || { echo "copilot CLI not found" >&2; exit 127; }
COPILOT_PATH="$(command -v copilot)"
if command -v node >/dev/null 2>&1; then
  COPILOT_CMD=("$COPILOT_PATH")
elif command -v powershell.exe >/dev/null 2>&1 && command -v wslpath >/dev/null 2>&1 &&
     [[ -f "${COPILOT_PATH}.ps1" ]]; then
  COPILOT_PS1="$(wslpath -w "${COPILOT_PATH}.ps1")"
  COPILOT_CMD=(powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$COPILOT_PS1")
else
  echo "copilot CLI found at $COPILOT_PATH, but neither node nor the Windows PowerShell shim is runnable" >&2
  exit 127
fi
mkdir -p .agent-runs
TS="$(date +%Y%m%d-%H%M%S)"; OUT=".agent-runs/${TS}-copilot-${AGENT}.md"
if [[ "${COPILOT_READ_ONLY:-1}" == "1" ]]; then
  MODE="read-only"
  GUARDRAIL="STRICTLY READ-ONLY. Do not edit, create, delete, rename, move, format, or generate files. Do not install dependencies or run commands that mutate git, the working tree, or external state. Use tools only to inspect and reason. If the task conflicts with these restrictions, report the conflict and stop."
else
  [[ -n "${COPILOT_WRITE_FILES:-}" ]] || { echo "Mutating task requires explicit COPILOT_WRITE_FILES (semicolon-separated absolute paths)" >&2; exit 2; }
  MODE="bounded-write"
  GUARDRAIL="BOUNDED MUTATING TASK. You may modify only these explicitly authorized absolute paths: ${COPILOT_WRITE_FILES}. Do not modify any other file or external state. Do not run git commit, reset, checkout, clean, restore, push, or other destructive commands. Stop and report if the task requires broader changes."
fi
PROMPT="${GUARDRAIL}

TASK:
${TASK}"
CMD=("${COPILOT_CMD[@]}" "--agent=${AGENT}" --allow-all-tools -p "$PROMPT")
[[ -z "${COPILOT_MODEL:-}" ]] || CMD+=("--model=${COPILOT_MODEL}")
{
  echo "# Copilot subagent: ${AGENT}"; echo; echo "- timestamp: ${TS}"; echo "- mode: ${MODE}"; echo "- task: ${TASK}"; echo; echo "## Response"; echo
  "${CMD[@]}"
} | tee "$OUT"
echo "Saved: $OUT" >&2
