#!/usr/bin/env bash

set -u
set -o pipefail

metric=${1:-rx}
os=$(uname -s 2>/dev/null || printf 'Unknown')

case "$metric" in
  udp-rx|udp-tx|tcp-rx|tcp-tx)
    if [[ "$os" == Darwin ]] && command -v nettop >/dev/null 2>&1; then
      protocol=${metric%%-*}
      direction=${metric##*-}
      field=bytes_in
      [[ "$direction" == tx ]] && field=bytes_out
      nettop -J "$field" -l 1 -m "$protocol" 2>/dev/null |
        awk '{ sum += $4 } END { printf "%.0f\n", sum }'
    else
      # Linux does not expose equivalent per-protocol byte totals through
      # /proc/net/dev. Keep the Sampler widget healthy when unavailable.
      printf '0\n'
    fi
    exit 0
    ;;
  rx|tx) direction=$metric ;;
  *) direction=rx ;;
esac

read_bytes() {
  case "$os" in
    Darwin)
      if command -v netstat >/dev/null 2>&1; then
        netstat -ibdn 2>/dev/null | awk -v direction="$direction" '
          $1 != "lo0" && $3 ~ /^<Link/ { sum += (direction == "rx" ? $7 : $10) }
          END { printf "%.0f\n", sum }
        '
      else
        printf '0\n'
      fi
      ;;
    Linux)
      if [[ -r /proc/net/dev ]]; then
        awk -v direction="$direction" '
          NR > 2 { gsub(/:/, "", $1); if ($1 != "lo") sum += (direction == "rx" ? $2 : $10) }
          END { printf "%.0f\n", sum }
        ' /proc/net/dev
      else
        printf '0\n'
      fi
      ;;
    *) printf '0\n' ;;
  esac
}

first=$(read_bytes)
sleep 0.5
second=$(read_bytes)

if [[ "$first" =~ ^[0-9]+$ && "$second" =~ ^[0-9]+$ && "$second" -ge "$first" ]]; then
  awk -v first="$first" -v second="$second" 'BEGIN { printf "%.0f\n", (second-first) * 2 }'
else
  printf '0\n'
fi

exit 0
