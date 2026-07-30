#!/usr/bin/env bash

set -u
set -o pipefail

os=$(uname -s 2>/dev/null || printf 'Unknown')

case "$os" in
  Darwin)
    if command -v ps >/dev/null 2>&1; then
      ps -A -o %cpu 2>/dev/null |
        awk '{ sum += $1 } END { printf "%.1f\n", sum }'
    else
      printf '0\n'
    fi
    ;;
  Linux)
    if [[ -r /proc/stat ]]; then
      read -r _ u1 n1 s1 i1 w1 q1 sq1 st1 _ < /proc/stat
      total1=$((u1+n1+s1+i1+w1+q1+sq1+st1))
      idle1=$((i1+w1))
      sleep 0.2
      read -r _ u2 n2 s2 i2 w2 q2 sq2 st2 _ < /proc/stat
      total2=$((u2+n2+s2+i2+w2+q2+sq2+st2))
      idle2=$((i2+w2))
      awk -v total="$((total2-total1))" -v idle="$((idle2-idle1))" \
        'BEGIN { if (total > 0) printf "%.1f\n", 100 * (total-idle) / total; else print "0" }'
    else
      printf '0\n'
    fi
    ;;
  *) printf '0\n' ;;
esac

exit 0
