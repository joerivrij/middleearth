#!/usr/bin/env bash

set -u
set -o pipefail

os=$(uname -s 2>/dev/null || printf 'Unknown')

case "$os" in
  Darwin)
    if command -v memory_pressure >/dev/null 2>&1; then
      memory_pressure 2>/dev/null |
        awk '/Pages free/ { gsub(/[^0-9]/, "", $3); print $3; found=1; exit } END { if (!found) print "0" }'
    else
      printf '0\n'
    fi
    ;;
  Linux)
    if [[ -r /proc/meminfo ]]; then
      awk '/MemFree:/ { printf "%.0f\n", $2 / 4; found=1; exit } END { if (!found) print "0" }' /proc/meminfo
    else
      printf '0\n'
    fi
    ;;
  *) printf '0\n' ;;
esac

exit 0
