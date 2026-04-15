#!/usr/bin/env bash
# functions-validate.sh — Cloud Functions lint, typecheck, test, build
# Usage: functions-validate.sh [typecheck|lint|test|build]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
FUNCTIONS_DIR="${REPO_ROOT}/functions"

cmd="${1:-test}"

cd "${FUNCTIONS_DIR}"
npm ci --prefer-offline 2>/dev/null || npm ci

case "${cmd}" in
  typecheck)
    echo "Running TypeScript type check..."
    npm run typecheck
    ;;
  lint)
    echo "Running ESLint..."
    npm run lint
    ;;
  test)
    echo "Running function tests..."
    npm test
    ;;
  build)
    echo "Building function zip artifact..."
    npm run build:zip
    ;;
  *)
    echo "Usage: functions-validate.sh [typecheck|lint|test|build]" >&2
    exit 1
    ;;
esac
