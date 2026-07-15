#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KUBECONFIG="${KUBECONFIG:-${ROOT}/machine/kubeconfig}"

for command in flux kubectl; do
  command -v "${command}" >/dev/null || {
    echo "missing required command: ${command}" >&2
    exit 1
  }
done

export KUBECONFIG
flux check --pre
flux install
kubectl apply -k "${ROOT}/cluster/flux-system"

echo "Flux is installed and following the public middleearth repository."
echo "After these files are on main, check it with: flux get all -A"
