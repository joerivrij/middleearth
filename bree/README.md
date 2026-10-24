# Bree

Bree provides a common Raspberry Pi toolbox, optional diagnostic tools, and
Bree's own Tailscale and Unbound DNS services. The Ansible entry points and
inventory live here; reusable roles live in Imladris.

| Profile | Purpose |
| --- | --- |
| `prancing-pony` | Shared Pi essentials: fzf, btop, ripgrep, fd, bat, tmux, ncdu, jq, curl, wget, git, unzip, lsof and tree. |
| `greenway --tags disks` | SMART, USB, disk I/O, SATA and NVMe diagnostics. |
| `greenway --tags network` | ip/ss, ping, dig, ethtool, tcpdump and traceroute. |
| `bree` | Shared toolbox plus tailscaled and Unbound. |
| `workstation` | Original local zsh, Starship and gopass setup. |

## Run from your controller

Install `ansible-core` 2.19 or newer on your controller. Targets need SSH,
Python 3, APT and sudo access. The package roles support current Raspberry Pi
OS, Debian and Ubuntu; they do not change Pi boot settings or reboot hosts.

The default inventory is `ansible/inventories/bree/hosts.yml`, targeting
`pi@bree`. Edit it for your SSH username/address. Run from the repository root:

```sh
# Preview, then install the shared essentials.
./bree/bootstrap.sh prancing-pony --check --diff
./bree/bootstrap.sh prancing-pony

# Choose diagnostics explicitly; without tags Greenway installs nothing.
./bree/bootstrap.sh greenway --tags disks
./bree/bootstrap.sh greenway --tags network
./bree/bootstrap.sh greenway --tags disks,network

# Provision Bree itself, including the common toolbox.
./bree/bootstrap.sh bree
```

Use `-K` if sudo requires a password, `--private-key PATH` for another SSH key,
or `-i PATH --limit HOST` for another inventory. Custom inventories should put
shared-toolbox targets in `raspberry_pis`; the service playbook targets only
the host named `bree`. `--list-hosts`, `--list-tasks` and `--syntax-check` inspect
a run without connecting. Check mode is a preview: package-dependent service
and validation tasks may need an initial real installation.

For a local Pi, use an inventory with the same group:

```yaml
all:
  children:
    raspberry_pis:
      hosts:
        localhost:
          ansible_connection: local
          ansible_python_interpreter: /usr/bin/python3
```

Save that as `/tmp/pi.yml` and run `./bree/pi-bootstrap.sh -i /tmp/pi.yml`.
`pi-bootstrap.sh` is a compatibility entry point for `prancing-pony`; it now
requires Ansible and uses the inventory rather than installing locally by default.
No diagnostic task formats disks, changes mounts or runs benchmarks. Package
installation may start the distro's packaged monitoring services.

Override `pi_toolbox_extra_packages` to add essentials, or replace
`pi_toolbox_packages`, `diagnostics_disk_packages` or
`diagnostics_network_packages` in inventory variables.

## Bree's services

The `bree` profile installs Tailscale from its signed APT repository and enables
`tailscaled`. Authentication is separate, using the existing helper:

```sh
./bree/tailscale-over-ssh.sh pi@bree
```

The helper reads `bree/tailscale/auth` from the local gopass store and sends it
over SSH. Set `GOPASS_ENTRY` or `SSH_KEY` to override those defaults. It skips
reading gopass when the target is already authenticated.

Unbound is a caching recursive resolver by default. Configure
`ansible/inventories/bree/host_vars/bree.yml` before exposing DNS to clients:

```yaml
unbound_interfaces:
  - 127.0.0.1
  - 192.168.1.10       # replace with Bree's stable LAN address
unbound_allowed_networks:
  - 127.0.0.0/8
  - 192.168.1.0/24     # replace with your client subnet
unbound_forwarders: [] # or upstream DNS IPs for forwarding
unbound_local_records:
  - 'bree.home.arpa. IN A 192.168.1.10'
```

It initially listens only on `127.0.0.1`. To use it over Tailscale, authenticate
first, add Bree's stable Tailscale IP and allowed client CIDRs, then rerun with
`./bree/bootstrap.sh bree --tags dns`. Listening on a specific tailnet IP also
requires that address to be available when Unbound starts after a reboot.
For hosts where that is unsuitable, use `0.0.0.0` with explicit client CIDRs
and firewall rules. Permit UDP and TCP port 53 for the intended clients and
configure the LAN DHCP server or tailnet DNS settings to use Bree. These
network-wide settings are not changed by the playbook. Recursive mode needs
outbound UDP/TCP 53; forwarding mode needs access to the chosen upstreams.

The role validates its rendered configuration before installing it and checks
the complete configuration before starting/restarting the service. It does
not change the host's own resolver or disable existing DNS services. Resolve
any existing port 53 listener conflict before enabling additional addresses.
Test on Bree with `dig @127.0.0.1 example.com` after installing network tools,
and repeat from a client against Bree's configured LAN/tailnet address.

## Interactive workstation setup

```sh
./bree/bootstrap.sh workstation
# or: bash ./bree/workstation-bootstrap.sh
```

This retains the original workstation bootstrap: it writes `~/.zshrc`, installs
Starship and gopass, and selects zsh as your login shell. Pi and service profiles
do not configure your shell. `eza` icons remain disabled by default; after
selecting a Nerd Font in your terminal, set `export EZA_ICONS=auto` in `~/.zshrc`.
