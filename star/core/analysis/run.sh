#!/usr/bin/env bash
set -euo pipefail

# Always run from the repository root so target paths resolve consistently.
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
cd "${repo_root}"

jobs="${CARGO_BUILD_JOBS:-$(nproc)}"
echo "cargo build -j ${jobs} --bin analysis --features timecop --release"
RUSTFLAGS="-C debuginfo=2 -C force-frame-pointers=yes" \
 cargo build -j "${jobs}" --bin analysis --features timecop --release

echo "valgrind ./target/release/analysis"
valgrind  --leak-check=full --show-leak-kinds=all ./target/release/analysis
