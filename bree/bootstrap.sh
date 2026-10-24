#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: bree/bootstrap.sh PROFILE [ansible-playbook options]

  prancing-pony  Common Raspberry Pi / Debian command-line tools
  greenway      Optional diagnostics: --tags disks,network
  bree          Bree's Tailscale and DNS services
  workstation   Local interactive shell setup (no Ansible)

Ansible targets pi@bree by default. Override with -i INVENTORY and --limit HOST.
Use --check --diff to preview, or -K to prompt for sudo.
EOF
}

profile="${1:---help}"
case "$profile" in
  -h|--help) usage; exit 0 ;;
  prancing-pony|greenway|bree|workstation) shift ;;
  *) usage >&2; exit 2 ;;
esac

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
if [[ "$profile" == workstation ]]; then
  exec bash "$script_dir/workstation-bootstrap.sh" "$@"
fi

if ! command -v ansible-playbook >/dev/null 2>&1; then
  echo 'Install ansible-core on this controller before running an Ansible profile.' >&2
  exit 1
fi
export ANSIBLE_CONFIG="$script_dir/ansible/ansible.cfg"
exec ansible-playbook "$script_dir/ansible/playbooks/$profile.yml" "$@"
