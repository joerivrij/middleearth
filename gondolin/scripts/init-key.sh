#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
key_home="${project_dir}/.keys/gnupg"
fingerprint_file="${project_dir}/.keys/fingerprint"
public_key="${project_dir}/keys/gondolin-sops.asc"
sops_config="${project_dir}/.sops.yaml"

if [ -e "$fingerprint_file" ] || [ -d "$key_home" ]; then
  echo "Gondolin key material already exists; refusing to overwrite it." >&2
  exit 1
fi

mkdir -p "$key_home" "$(dirname -- "$public_key")"
chmod 700 "$key_home"

GNUPGHOME="$key_home" gpg --batch --passphrase '' \
  --quick-generate-key \
  'Gondolin SOPS <gondolin-sops@middleearth.invalid>' \
  future-default default 0

fingerprint=$(
  GNUPGHOME="$key_home" gpg --batch --with-colons \
    --list-secret-keys 'gondolin-sops@middleearth.invalid' |
    awk -F: '$1 == "fpr" { print $10; exit }'
)

if [ -z "$fingerprint" ]; then
  echo "Could not determine the Gondolin key fingerprint." >&2
  exit 1
fi

printf '%s\n' "$fingerprint" > "$fingerprint_file"
chmod 600 "$fingerprint_file"

GNUPGHOME="$key_home" gpg --batch --armor \
  --export "$fingerprint" > "$public_key"

config_tmp=$(mktemp "${TMPDIR:-/tmp}/gondolin-sops-config.XXXXXX")
trap 'rm -f "$config_tmp"' EXIT HUP INT TERM
awk -v fingerprint="$fingerprint" '
  /^[[:space:]]*pgp:/ {
    sub(/pgp:.*/, "pgp: " fingerprint)
  }
  { print }
' "$sops_config" > "$config_tmp"
install -m 0644 "$config_tmp" "$sops_config"

echo "Created isolated Gondolin SOPS key: $fingerprint"
echo "Updated .sops.yaml and exported the public key."
