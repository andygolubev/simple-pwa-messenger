#!/usr/bin/env bash
# rust-validate.sh — Rust crate lint, test, and WASM build
# Usage: rust-validate.sh [test|check-wasm32|build-wasm]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
CRATE_DIR="${REPO_ROOT}/crates/messenger-crypto"

cmd="${1:-test}"

cd "${CRATE_DIR}"

case "${cmd}" in
  test)
    echo "Running Rust tests..."
    cargo test
    ;;
  check-wasm32)
    echo "Checking compilation for wasm32-unknown-unknown target..."
    cargo check --target wasm32-unknown-unknown
    ;;
  build-wasm)
    echo "Building WASM module..."
    ./build-wasm.sh
    ;;
  *)
    echo "Usage: rust-validate.sh [test|check-wasm32|build-wasm]" >&2
    exit 1
    ;;
esac
