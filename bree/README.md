# Bree

Bree is the workstation bootstrap for Middle-earth.

It prepares a fresh Linux workstation with the common command-line tools,
shell configuration, Starship, and gopass needed to work on the labs.

```sh
./bree/bootstrap.sh
```

`eza` icons are disabled by default. They use Nerd Font-only glyphs, and the
font is selected by the terminal displaying the SSH session rather than by the
remote shell. After configuring a Nerd Font in that terminal, add
`export EZA_ICONS=auto` to `~/.zshrc`.

Tailscale is deliberately provisioned separately. The following command reads
the auth key from `odysseia/tailscale/auth` in the local gopass store and sends
it over SSH without writing it to disk:

```sh
./bree/tailscale-over-ssh.sh pi@bree
```

Set `SSH_KEY=/path/to/key` when the target needs a non-default SSH identity.
The script exits without opening gopass when the remote host is already
authenticated with Tailscale.

This project configures the operator's workstation. Reusable remote-host
provisioning belongs in Imladris.
