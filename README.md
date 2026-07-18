# Middleearth

One repository to rule the homelab.

`middleearth` is the container for everything I need to build, rebuild, and
maintain my homelab systems. It includes machine setup, infrastructure, and
small experiments that make the systems easier to understand. Over time this
will grow into the full map for my homelab:
hosts, services, setup notes, automation, and whatever other spells are needed
to bring the machines back from bare metal.

## The Map

```text
middleearth/
├── bree/
│   └── bootstrap.sh
├── grey-havens/
│   └── Go life-of-a-packet laboratory
├── erebor/
│   └── homelab infrastructure
├── LICENSE
└── README.md
```

## Bree

`bree` is the first stop on the road: a bootstrap script for a fresh Linux box.
It installs the basic tools I want everywhere, writes a practical `zsh`
configuration, installs Starship, and prepares Tailscale so the machine can join
the tailnet.

What it currently sets up:

- `zsh`, `git`, `curl`, `fzf`, `ripgrep`, `bat`, `tmux`, `direnv`, `htop`,
  `jq`, `unzip`, and certificates
- optional nicer shell tools like `eza` and `zoxide`, when available
- Starship prompt
- a fresh `~/.zshrc`
- Tailscale and the `tailscaled` service

Run it from the repository root:

```sh
./bree/bootstrap.sh
```

After the script finishes, Tailscale still needs the manual login step:

```sh
sudo tailscale up
```

Then restart the SSH session or run:

```sh
exec zsh
```

## Grey Havens

`grey-havens` is an interactive life-of-a-packet experiment. Its small Go
service traces the DNS, TCP, TLS, and HTTP stages involved in fetching a URL and
shows the resulting timeline in a local web page.

```sh
cd grey-havens
go run .
```

Then visit <http://127.0.0.1:8080>. See [grey-havens/README.md](grey-havens/README.md)
for details and possible next experiments.

## Future Realms

This repository is intended to become the source of truth for the full homelab.
Possible future inhabitants:

- host bootstrap scripts
- service definitions
- network and storage notes
- deployment automation
- recovery instructions
- secrets handling documentation
- inventory for every small box, old laptop, cursed adapter, and noble server

For now, the road begins in Bree.

## License

MIT. See [LICENSE](LICENSE).
