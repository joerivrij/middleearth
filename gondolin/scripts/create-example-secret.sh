#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
key_home="${project_dir}/.keys/gnupg"
fingerprint_file="${project_dir}/.keys/fingerprint"
output="${project_dir}/secrets/example/example-secret.enc.yaml"
plaintext=$(mktemp "${TMPDIR:-/tmp}/gondolin-secret.XXXXXX")
trap 'rm -f "$plaintext"' EXIT HUP INT TERM

if [ ! -f "$fingerprint_file" ]; then
  echo "Run make init-key first." >&2
  exit 1
fi

username=${GONDOLIN_EXAMPLE_USERNAME:-fellowship}
if [ -z "${GONDOLIN_EXAMPLE_TOKEN:-}" ]; then
  echo "Set GONDOLIN_EXAMPLE_TOKEN from the authoritative secret store." >&2
  exit 1
fi

kubectl create secret generic gondolin-example \
  --namespace gondolin \
  --from-literal="username=${username}" \
  --from-literal="token=${GONDOLIN_EXAMPLE_TOKEN}" \
  --dry-run=client --output=yaml > "$plaintext"

cd "$project_dir"
GNUPGHOME="$key_home" sops --config .sops.yaml encrypt \
  --filename-override secrets/example/example-secret.enc.yaml \
  --output "$output" \
  "$plaintext"

echo "Wrote SOPS-encrypted Secret to $output"
