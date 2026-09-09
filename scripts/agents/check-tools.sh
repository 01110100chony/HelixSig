#!/usr/bin/env bash
set -u
for cmd in git copilot agy jq; do
  if command -v "$cmd" >/dev/null 2>&1; then echo "[OK] $cmd -> $(command -v "$cmd")"; else echo "[MISSING] $cmd"; fi
done
if command -v agy >/dev/null 2>&1; then echo; echo "== agy models =="; agy models || true; echo; echo "== agy agents =="; agy agents || true; fi
