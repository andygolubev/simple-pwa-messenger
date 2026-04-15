#!/usr/bin/env bash
# Build the messenger-crypto crate as a WASM module and output to apps/pwa/pkg.
# Requires wasm-pack to be installed: https://rustwasm.github.io/wasm-pack/
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
OUTPUT_DIR="${REPO_ROOT}/apps/pwa/pkg"

echo "Building messenger-crypto WASM..."
cd "${SCRIPT_DIR}"

wasm-pack build \
  --target web \
  --out-dir "${OUTPUT_DIR}" \
  --release

echo "WASM artifact written to: ${OUTPUT_DIR}"
echo "Files:"
ls -lh "${OUTPUT_DIR}"
