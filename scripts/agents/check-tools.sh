#!/usr/bin/env bash
set -u
for cmd in git jq; do
  if command -v "$cmd" >/dev/null 2>&1; then echo "[OK] $cmd -> $(command -v "$cmd")"; else echo "[MISSING] $cmd"; fi
done
if command -v copilot >/dev/null 2>&1; then
  copilot_path="$(command -v copilot)"
  if command -v node >/dev/null 2>&1; then
    echo "[OK] copilot -> $copilot_path (native node)"
  elif command -v powershell.exe >/dev/null 2>&1 && command -v wslpath >/dev/null 2>&1 &&
       [[ -f "${copilot_path}.ps1" ]]; then
    echo "[OK] copilot -> ${copilot_path}.ps1 (Windows PowerShell interop)"
  else
    echo "[BROKEN] copilot -> $copilot_path (node unavailable and no runnable PowerShell shim)"
  fi
else
  echo "[MISSING] copilot"
fi
if command -v agy >/dev/null 2>&1; then
  agy_bin="$(command -v agy)"
elif command -v agy.exe >/dev/null 2>&1; then
  agy_bin="$(command -v agy.exe)"
else
  agy_bin=""
fi
if [[ -n "$agy_bin" ]]; then
  echo "[OK] agy -> $agy_bin"
  echo
  echo "== agy models =="
  "$agy_bin" models || true
  echo
  echo "== agy agents =="
  "$agy_bin" agents || true
else
  echo "[MISSING] agy (checked agy and agy.exe)"
fi
