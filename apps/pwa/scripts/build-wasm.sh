#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"

wasm-pack build "${PROJECT_ROOT}/crates/messenger-crypto" \
  --target web \
  --release \
  --out-dir "${PROJECT_ROOT}/apps/pwa/pkg"
