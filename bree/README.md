# Bree

Bree provides a common Raspberry Pi toolbox, optional diagnostic tools, and
Bree's own Tailscale and Unbound DNS services. The Ansible entry points and
inventory live here; reusable roles live in Imladris.

| Profile | Purpose |
| --- | --- |
| `prancing-pony` | Shared Pi essentials: fzf, btop, ripgrep, fd, bat, tmux, ncdu, jq, curl, wget, git, unzip, lsof and tree, plus Zsh and the Starship prompt. |
| `greenway --tags disks` | SMART, USB, disk I/O, SATA and NVMe diagnostics. |
| `greenway --tags network` | ip/ss, ping, dig, ethtool, tcpdump and traceroute. |
| `bree` | Shared toolbox, gopass, Kubernetes access tools, Tailscale subnet routing and dnsmasq. |
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

Prancing Pony installs Zsh, makes it the SSH user's login shell, and installs
Starship into `/usr/local/bin` using its official installer (only when absent).
The first installation needs HTTPS access to starship.rs and GitHub.
It configures the user's Bash and Zsh startup files with a managed block,
preserving other settings. Zsh gets history, completion and Emacs keybindings.
It enables fzf's Ctrl-R history search, Ctrl-T file selection and Alt-C
directory selection, adds `ll`, `..` and `...`, and makes Debian's `batcat` and
`fdfind` available as `bat` and `fd`. Reconnect after provisioning, or run
`exec zsh` to switch an existing session. Run `btop` for the process monitor;
existing tools such as htop are retained. Midnight Commander (`mc`) from the
previous toolbox is removed on the next run.
Set `pi_toolbox_configure_shell: false` to skip shell setup and Starship.
For Starship's default icons, select a Nerd Font in your client terminal.
The `bree` profile includes this same toolbox and shell configuration.

## Bree's access tools

The Bree profile adds gopass from its official signed APT repository, GnuPG,
terminal pinentry, kubectl, Krew, and the distro's `kubectx` package (which also
provides `kubens`). Krew is installed per user under `~/.krew` from the official
release, with its archive checksum verified; `bree_access_krew_version` pins
the bootstrap installer version. Subsequent upgrades use `kubectl krew upgrade`.
Its `bin` directory is added to Bash and Zsh's PATH. Both shells get the `k`
alias and `GPG_TTY` for terminal passphrase prompts; Zsh also gets kubectl
completion. Install or update just these tools with:

```sh
./bree/bootstrap.sh bree --tags access
```

`bree_access_kubectl_minor` selects the Kubernetes APT release stream, defaulting
to `v1.35` for the existing Hellas cluster. Keep it within one minor release of
both cluster API servers; verify Hellenistike's version before using it there.
The intended contexts are `hellenistike-odysseia` and `hellas-odysseia`.

Credentials are a separate recovery step. To reuse the existing gopass system,
import the shared GPG key and clone the same remote password store on Bree.
For example, from an already configured controller, replacing `FINGERPRINT`:

```sh
gpg --export-secret-keys FINGERPRINT | ssh pi@bree 'gpg --import'
```

Then on Bree, use the actual existing password-store Git URL:

```sh
gopass clone YOUR_EXISTING_STORE_GIT_URL
gopass ls
```

Git authentication to that remote must also be configured. If GPG asks about
recipient trust, verify the imported fingerprint and set its ownertrust with
`gpg --edit-key FINGERPRINT` and `trust`. Retain the store and key backups on
your other machines so Bree can be rebuilt independently.

To import a prepared, self-contained kubeconfig from the controller, opt in:

```sh
./bree/bootstrap.sh bree --tags access \
  -e bree_access_kubeconfig_src=/absolute/path/to/prepared-kubeconfig
```

This replaces the SSH user's `~/.kube/config` with mode `0600`, suppressing its
contents from Ansible logs and diffs. Use a kubeconfig containing the intended
contexts with embedded certificates/keys; external credential plugins must be
installed separately. The default empty source leaves existing credentials
untouched. Never put the kubeconfig or GPG private key in the repository.
Reconnect after provisioning, then use `kubectx`, `kubens`, or, explicitly:

```sh
kubectl --context hellas-odysseia get namespaces
kubectl --context hellenistike-odysseia get namespaces
kubectl krew version
```

This configures commands run on Bree; laptop SSH jump-host settings and DNS
migration are separate steps.

## Bree's services

The `bree` profile installs Tailscale from its signed APT repository and enables
`tailscaled`. Authentication is separate, using the existing helper:

```sh
./bree/tailscale-over-ssh.sh pi@bree
```

The helper reads `bree/tailscale/auth` from the local gopass store and sends it
over SSH. Set `GOPASS_ENTRY` or `SSH_KEY` to override those defaults. It skips
reading gopass when the target is already authenticated.

Bree now runs dnsmasq instead of Unbound. Shared LAN records live in
[`dns/hosts`](dns/hosts), with DNS served at `192.168.1.151` on the LAN and
`100.115.75.42` on Tailscale. It also advertises routes for the seven cluster
node IPs, so those nodes do not need Tailscale installed.

```sh
./bree/bootstrap.sh bree --tags dns,subnet
```

Client DNS settings, tailnet route approval, and access rules are separate.
See [DNS and subnet routing](dns/README.md) for rollout, verification and rollback.

## Interactive workstation setup

```sh
./bree/bootstrap.sh workstation
# or: bash ./bree/workstation-bootstrap.sh
```

This retains the original workstation bootstrap: it writes `~/.zshrc`, installs
Starship and gopass, and selects zsh as your login shell. The Ansible Pi and Bree
profiles now configure Zsh and Starship too, while preserving existing startup
files. `eza` icons in the workstation script remain disabled by default; after
selecting a Nerd Font in your terminal, set `export EZA_ICONS=auto` in `~/.zshrc`.
