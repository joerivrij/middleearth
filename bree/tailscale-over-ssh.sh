#!/usr/bin/env bash
set -euo pipefail

readonly TARGET="${1:-pi@bree}"
readonly GOPASS_ENTRY="${GOPASS_ENTRY:-bree/tailscale/auth}"

ssh_options=(-o BatchMode=yes)
if [[ -n "${SSH_KEY:-}" ]]; then
  ssh_options+=(-i "$SSH_KEY")
fi

if ssh "${ssh_options[@]}" "$TARGET" \
  'command -v tailscale >/dev/null 2>&1 && sudo tailscale ip -4 >/dev/null 2>&1'
then
  echo "Tailscale is already authenticated on $TARGET."
  exit 0
fi

for command_name in gopass ssh; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Required local command is missing: $command_name" >&2
    exit 1
  fi
done

auth_key="$(gopass show -o "$GOPASS_ENTRY")"
if [[ -z "$auth_key" || "$auth_key" == *$'\n'* ]]; then
  echo "The gopass entry must contain exactly one non-empty line." >&2
  exit 1
fi

{
  printf '%s\n' "$auth_key"
  cat <<'REMOTE_SCRIPT'
set -euo pipefail

if ! command -v tailscale >/dev/null 2>&1; then
  curl -fsSL https://tailscale.com/install.sh | sh
fi

sudo systemctl enable --now tailscaled

if ! sudo tailscale ip -4 >/dev/null 2>&1; then
  sudo tailscale up --auth-key="$auth_key"
fi

unset auth_key
sudo tailscale status
REMOTE_SCRIPT
} | ssh "${ssh_options[@]}" "$TARGET" \
  'IFS= read -r auth_key; export auth_key; exec bash -s'

unset auth_key
