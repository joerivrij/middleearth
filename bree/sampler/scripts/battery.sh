#!/usr/bin/env bash

set -u
set -o pipefail

mode=${1:-status}
os=$(uname -s 2>/dev/null || printf 'Unknown')
percent=''
state='Unavailable'

case "$os" in
  Darwin)
    if command -v pmset >/dev/null 2>&1; then
      battery=$(pmset -g batt 2>/dev/null || true)
      percent=$(printf '%s\n' "$battery" | awk 'match($0, /[0-9]+%/) { value=substr($0, RSTART, RLENGTH-1); print value; exit }')
      state=$(printf '%s\n' "$battery" | awk -F';' 'NF >= 2 { gsub(/^[[:space:]]+|[[:space:]]+$/, "", $2); print $2; exit }')
      [[ -n "$state" ]] || state='Unavailable'
    fi
    ;;
  Linux)
    for supply in /sys/class/power_supply/BAT*; do
      [[ -d "$supply" ]] || continue
      [[ -r "$supply/capacity" ]] && percent=$(<"$supply/capacity")
      [[ -r "$supply/status" ]] && state=$(<"$supply/status")
      break
    done
    ;;
esac

case "$mode" in
  percent)
    [[ "$percent" =~ ^[0-9]+$ ]] && printf '%s\n' "$percent" || printf '0\n'
    ;;
  *)
    if [[ "$percent" =~ ^[0-9]+$ ]]; then
      printf '%s%% — %s\n' "$percent" "$state"
    else
      printf 'Battery unavailable\n'
    fi
    ;;
esac

exit 0
