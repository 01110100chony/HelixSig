#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
gate="${1:-H0}"
case "$gate" in H0|H1|H2|H3|H4|H5) ;; *) echo "Gate $gate is not implemented" >&2; exit 1;; esac
python_bin="${HELIX_PYTHON:-python3}"
cmake -S cpp -B build/native -DCMAKE_BUILD_TYPE=Release
cmake --build build/native -j 2
ctest --test-dir build/native --output-on-failure
mkdir -p artifacts
corpus="$(mktemp -d artifacts/verify-XXXXXX)/corpus"
"$python_bin" scripts/generate_data.py --out "$corpus"
"$python_bin" scripts/validate.py --native build/native/kernel_driver --corpus "$corpus"
cmake -S cpp -B build/sanitized -DCMAKE_CXX_COMPILER=clang++ -DCMAKE_BUILD_TYPE=Debug -DHELIX_SANITIZERS=ON
cmake --build build/sanitized -j 2
ASAN_OPTIONS=detect_leaks=1 UBSAN_OPTIONS=halt_on_error=1 ctest --test-dir build/sanitized --output-on-failure
ASAN_OPTIONS=detect_leaks=1 UBSAN_OPTIONS=halt_on_error=1 "$python_bin" scripts/validate.py --native build/sanitized/kernel_driver --corpus "$corpus"
find cpp -type f \( -name '*.cpp' -o -name '*.hpp' \) -print0 | xargs -0 clang-format --dry-run --Werror
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
echo "$gate automated checks PASS; independent review still required"
