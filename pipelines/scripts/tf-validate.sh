#!/usr/bin/env bash
# tf-validate.sh — Terraform format check and validation
# Usage: tf-validate.sh [fmt|validate]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
INFRA_DIR="${REPO_ROOT}/infra"

cmd="${1:-fmt}"

cd "${INFRA_DIR}"

case "${cmd}" in
  fmt)
    echo "Running: terraform fmt -check -recursive"
    terraform fmt -check -recursive
    ;;
  validate)
    echo "Running: terraform init (backend=false) + validate"
    terraform init -backend=false -input=false
    terraform validate
    ;;
  *)
    echo "Usage: tf-validate.sh [fmt|validate]" >&2
    exit 1
    ;;
esac
