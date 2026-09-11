#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
requested_gate="${1:-H6}"
gate="$requested_gate"
# H6 consolidates the complete existing suite; smoke is never performance evidence.
if [[ "$gate" == "H6" ]]; then
  scripts/validate_release.sh
  gate=H5
fi
case "$gate" in H0|H1|H2|H3|H4|H5) ;; *) echo "Gate $gate is not implemented" >&2; exit 1;; esac
python_bin="${HELIX_PYTHON:-python3}"
cmake -S cpp -B build/native -DCMAKE_CXX_COMPILER="${CXX:-g++}" -DCMAKE_BUILD_TYPE=Release \
  -DBUILD_TESTING=ON -DHELIX_SANITIZERS=OFF -DHELIX_BENCHMARKS=OFF
cmake --build build/native -j 2
ctest --test-dir build/native --output-on-failure --no-tests=error
mkdir -p artifacts
corpus="$(mktemp -d artifacts/verify-XXXXXX)/corpus"
"$python_bin" scripts/generate_data.py --out "$corpus"
"$python_bin" scripts/validate.py --native build/native/kernel_driver --corpus "$corpus"
cmake -S cpp -B build/sanitized -DCMAKE_CXX_COMPILER=clang++-18 -DCMAKE_BUILD_TYPE=Debug \
  -DBUILD_TESTING=ON -DHELIX_SANITIZERS=ON -DHELIX_BENCHMARKS=OFF
cmake --build build/sanitized -j 2
ASAN_OPTIONS=detect_leaks=1 UBSAN_OPTIONS=halt_on_error=1 ctest --test-dir build/sanitized --output-on-failure --no-tests=error
ASAN_OPTIONS=detect_leaks=1 UBSAN_OPTIONS=halt_on_error=1 "$python_bin" scripts/validate.py --native build/sanitized/kernel_driver --corpus "$corpus"
find cpp -type f \( -name '*.cpp' -o -name '*.hpp' \) -print0 | xargs -0 clang-format-18 --dry-run --Werror
if [[ "$gate" != "H0" ]]; then
  cargo fmt --check
  cargo clippy --all-targets --locked -j 2 -- -D warnings
  timeout 120s cargo test --locked -j 2
fi
if [[ "$gate" == "H2" || "$gate" == "H3" || "$gate" == "H4" || "$gate" == "H5" ]]; then
  cargo build --locked -j 2
  "$python_bin" scripts/validate_pipeline.py --binary target/debug/helix --corpus "$corpus"
fi
if [[ "$gate" == "H3" || "$gate" == "H4" || "$gate" == "H5" ]]; then
  "$python_bin" scripts/validate_concurrent.py --binary target/debug/helix --corpus "$corpus"
fi
if [[ "$gate" == "H4" || "$gate" == "H5" ]]; then
  "$python_bin" scripts/validate_failures.py --binary target/debug/helix --corpus "$corpus"
fi
if [[ "$gate" == "H5" ]]; then
  "$python_bin" scripts/validate_experiments.py
  smoke_root="$(mktemp -d artifacts/h5-smoke-XXXXXX)"
  "$python_bin" scripts/experiment.py --smoke --out "$smoke_root/campaign"
fi
echo "$requested_gate automated checks PASS; independent review and human approval remain separate"
