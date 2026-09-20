#!/usr/bin/env bash
# A small toolbox for Raspberry Pi OS / Debian / Ubuntu.
# Run with: bash pi-toolbox.sh
set -euo pipefail

if ! command -v apt-get >/dev/null 2>&1; then
  echo 'This script requires a Debian-based OS with apt-get.' >&2
  exit 1
fi

SUDO=()
if (( EUID != 0 )); then
  if ! command -v sudo >/dev/null 2>&1; then
    echo 'Run this script as root, or install sudo first.' >&2
    exit 1
  fi
  SUDO=(sudo)
  sudo -v
fi

packages=(
  btop          # CPU, memory, disk and process monitor
  fzf           # Interactive fuzzy finder
  ripgrep       # Fast text search: rg
  fd-find       # Friendly file search: fdfind
  bat           # Syntax-highlighted file viewer: batcat
  tmux          # Terminal sessions that survive SSH disconnects
  ncdu          # Interactive disk usage browser
  jq            # Read and filter JSON
  curl wget git unzip ca-certificates
  smartmontools # SSD health: smartctl
  usbutils      # USB devices: lsusb
  lsof          # Open files and the processes using them
  iotop         # Disk activity by process (usually needs sudo)
  sysstat       # I/O statistics: iostat
  iproute2      # Network interfaces, routes and sockets: ip, ss
  iputils-ping
  dnsutils      # DNS troubleshooting: dig
  ethtool       # Ethernet link diagnostics
  tree          # Directory tree viewer
)

echo 'Refreshing package lists...'
"${SUDO[@]}" apt-get update
echo 'Installing the Pi toolbox...'
"${SUDO[@]}" apt-get install -y --no-install-recommends "${packages[@]}"

cat <<'EOF'

Pi toolbox installed!

  btop                      System overview
  ncdu -x ~                 Explore space used in your home directory
  lsblk -f                  Disks, filesystems and mount points
  sudo smartctl -a /dev/sda  SSD health (check the device name first)
  sudo iotop -o             Processes currently doing disk I/O
  iostat -xz 1              Disk statistics, updated every second
  ip -br addr               Network addresses
  ss -tulpn                 Listening ports
  dig example.com           DNS lookup
  fdfind . ~ | fzf          Search and select a file path
  batcat /etc/os-release    View a file with highlighting
  tmux new -s work          Start a persistent terminal session

Detach from tmux with Ctrl-b, then d. Reconnect with: tmux attach -t work
On Debian-based systems, bat and fd are commonly named batcat and fdfind.

No disks were formatted and no shell configuration was changed.
EOF
