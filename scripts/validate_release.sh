#!/usr/bin/env bash
# Offline regression: failed reviewers must never produce a successful group.
set -euo pipefail
cd "$(dirname "$0")/.."
for script in scripts/*.sh scripts/agents/*.sh; do
  bash -n "$script"
done
for profile in independent-reviewer architecture-critic benchmark-critic; do
  test -s "agent-prompts/antigravity/$profile.md"
done
mkdir -p artifacts
test_root="$(mktemp -d artifacts/release-tools-XXXXXX)"
mkdir -p "$test_root/scripts/agents"
cp scripts/agents/{parallel-review,release-review}.sh "$test_root/scripts/agents/"
cat > "$test_root/scripts/agents/copilot-agent.sh" <<'EOF'
#!/usr/bin/env bash
[[ "$1" != "${FAIL_REVIEWER:-}" ]] || exit 23
EOF
cat > "$test_root/scripts/agents/agy-agent.sh" <<'EOF'
#!/usr/bin/env bash
[[ "$1" != "${FAIL_REVIEWER:-}" ]] || exit 23
EOF
chmod +x "$test_root/scripts/agents/"*.sh
(
  cd "$test_root"
  for group in parallel release; do
    FAIL_REVIEWER= scripts/agents/"$group"-review.sh offline-test > "$group-success.log" 2>&1
    if [[ "$group" == parallel ]]; then
      reviewers=(cpp-reviewer rust-concurrency-reviewer test-adversary)
    else
      reviewers=(cpp-reviewer rust-concurrency-reviewer benchmark-auditor independent-reviewer)
    fi
    for reviewer in "${reviewers[@]}"; do
      log="$group-$reviewer.log"
      if FAIL_REVIEWER="$reviewer" scripts/agents/"$group"-review.sh offline-test > "$log" 2>&1; then
        echo "FAIL: $group masked $reviewer's nonzero exit" >&2
        exit 1
      fi
      if grep -qi 'reviews complete' "$log"; then
        echo "FAIL: $group reported completion after $reviewer failed" >&2
        exit 1
      fi
    done
  done
)
echo "Release tooling PASS: two successful groups and seven individual reviewer failures; no external CLI called"
