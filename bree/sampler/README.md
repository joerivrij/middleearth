# Local system dashboard

This is a deliberately small experiment with [Sampler](https://github.com/sqshq/sampler). It reads only local system information and does not run or depend on monitoring infrastructure.

## Install on macOS

Install Sampler with Homebrew:

```bash
brew install sampler
```

MacPorts also packages it as `sampler`:

```bash
sudo port install sampler
```

Sampler's latest upstream release is old, so treat this dashboard as a convenience rather than critical monitoring.

## Run

Start the dashboard in a terminal at least 120 columns wide:

```bash
sampler -c ~/go/src/github.com/joerivrij/middleearth/bree/sampler/system.yml
```

For regular use, add this to `~/.zshrc` (or the startup file for your shell):

```bash
alias sampler-system='sampler -c ~/go/src/github.com/joerivrij/middleearth/bree/sampler/system.yml'
```

Then open a new shell and run `sampler-system`.

## Commands

Sampler and Bash are required. The dashboard otherwise uses native commands normally present on macOS and common Linux distributions: `uname`, `hostname`, `uptime`, `df`, `ps`, `awk`, `head`, and `sleep`.

Optional platform commands and data sources are:

- macOS: `top` for CPU, `memory_pressure` for memory, `netstat` for network counters, and `pmset` for battery data.
- Linux: `/proc/stat`, `/proc/meminfo`, and `/proc/net/dev` for CPU, memory, and network counters; `/sys/class/power_supply/BAT*` for battery data.

If an optional metric is unavailable, its numeric script prints `0`; the battery state says `Battery unavailable`. This keeps Sampler from continuously reporting command errors.

## Platform differences

- CPU follows Sampler's macOS example by summing per-process CPU from `ps`; on multicore systems it can exceed 100%. Linux calculates total non-idle CPU over a short interval from `/proc/stat`.
- Memory shows free pages from `memory_pressure` on macOS. Linux estimates free 4 KiB pages from `MemFree` in `/proc/meminfo`.
- The dashboard's network bars use `nettop` on macOS to split UDP/TCP receive and transmit bytes. Linux has no straightforward equivalent per-protocol byte counters, so these four bars gracefully return zero there. The network script also retains aggregate `rx`/`tx` modes backed by macOS `netstat` and Linux `/proc/net/dev` for future use.
- Battery uses `pmset` on macOS and the first `BAT*` power supply on Linux. Desktops and machines without an exposed battery show the documented fallback.
- Process sorting uses BSD `ps` flags on macOS and procps sorting on Linux.
- Load average, root disk usage, uptime, hostname, OS, and kernel use portable commands directly from `system.yml`.

The fixed widget positions are designed for an 80-column terminal. Sampler allows repositioning widgets interactively if a different layout fits your terminal better.
