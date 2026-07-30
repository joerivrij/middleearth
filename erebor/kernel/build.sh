#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KERNEL_VERSION="${KERNEL_VERSION:-6.18.15}"
IMAGE="${KERNEL_IMAGE:-local/erebor-kernel:${KERNEL_VERSION}}"
PROBE="erebor-kernel-config-probe"
ARTIFACT="erebor-kernel-artifact"
MACHINE_MODULES="${ROOT}/../machine/modules"

command -v container >/dev/null || {
  echo "missing required command: container" >&2
  exit 1
}

container system start

if [[ ! -f "${ROOT}/base.config" ]]; then
  # An OCI image has no kernel of its own. Start a disposable VM with Apple's
  # default kernel and capture its embedded config as our compatible baseline.
  container run --rm --name "${PROBE}" debian:13 \
    sh -c 'zcat /proc/config.gz' > "${ROOT}/base.config"
fi

container build \
  --tag "${IMAGE}" \
  --build-arg "KERNEL_VERSION=${KERNEL_VERSION}" \
  --file "${ROOT}/Containerfile" \
  "${ROOT}"

container rm --force "${ARTIFACT}" >/dev/null 2>&1 || true
container create --name "${ARTIFACT}" "${IMAGE}"
container start "${ARTIFACT}"
container cp "${ARTIFACT}:/Image" "${ROOT}/Image"
rm -rf "${MACHINE_MODULES}"
container cp "${ARTIFACT}:/modules/lib/modules" "${MACHINE_MODULES}"
container rm --force "${ARTIFACT}"
chmod 0644 "${ROOT}/Image"

echo "custom Erebor kernel built at ${ROOT}/Image"
echo "matching kernel modules staged at ${MACHINE_MODULES}"
