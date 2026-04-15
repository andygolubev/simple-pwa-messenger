#!/usr/bin/env bash
# pwa-validate.sh — PWA lint, typecheck, test, build
# Usage: pwa-validate.sh [typecheck|lint|test|build]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
PWA_DIR="${REPO_ROOT}/apps/pwa"

cmd="${1:-test}"

cd "${PWA_DIR}"
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
    echo "Running PWA tests..."
    npm test
    ;;
  build)
    echo "Building PWA..."
    npm run build
    ;;
  *)
    echo "Usage: pwa-validate.sh [typecheck|lint|test|build]" >&2
    exit 1
    ;;
esac
