#!/usr/bin/env bash
# Compatibility entry point for the shared Raspberry Pi toolbox.
set -euo pipefail
exec "$(dirname "$0")/bootstrap.sh" prancing-pony "$@"
