#!/usr/bin/env bash
set -euo pipefail

echo "== Bree bootstrap =="

echo "== Updating apt =="
sudo apt update

echo "== Installing terminal tools =="
sudo apt install -y \
  zsh \
  git \
  curl \
  fzf \
  ripgrep \
  bat \
  tmux \
  direnv \
  htop \
  jq \
  gopass \
  unzip \
  ca-certificates

echo "== Installing optional nicer tools if available =="
sudo apt install -y eza zoxide || true

echo "== Installing Starship prompt =="
if ! command -v starship >/dev/null 2>&1; then
  curl -sS https://starship.rs/install.sh | sh -s -- -y
fi

echo "== Writing ~/.zshrc =="
cat > "$HOME/.zshrc" <<'EOF'
# History
HISTFILE=~/.zsh_history
HISTSIZE=100000
SAVEHIST=100000
setopt appendhistory
setopt sharehistory
setopt hist_ignore_dups
setopt hist_ignore_space
setopt hist_reduce_blanks

# Completion
autoload -Uz compinit
compinit
zstyle ':completion:*' menu select

# Keybindings
bindkey -e
bindkey '^R' history-incremental-search-backward

# fzf
[ -f /usr/share/doc/fzf/examples/key-bindings.zsh ] && source /usr/share/doc/fzf/examples/key-bindings.zsh
[ -f /usr/share/doc/fzf/examples/completion.zsh ] && source /usr/share/doc/fzf/examples/completion.zsh

# zoxide
command -v zoxide >/dev/null && eval "$(zoxide init zsh)"

# starship
command -v starship >/dev/null && eval "$(starship init zsh)"

# Aliases
alias ll='ls -lah'
alias grep='grep --color=auto'
alias k='kubectl'
alias g='git'
alias ..='cd ..'
alias ...='cd ../..'

# eza's icons require a Nerd Font in the terminal that displays this shell.
# SSH cannot configure that client-side font, so keep icons off by default.
# Set EZA_ICONS=auto after selecting a Nerd Font locally.
command -v eza >/dev/null && alias ls='eza --icons="${EZA_ICONS:-never}" --group-directories-first'
command -v batcat >/dev/null && alias cat='batcat --paging=never'
command -v bat >/dev/null && alias cat='bat --paging=never'
EOF

echo "== Setting zsh as default shell =="
if [ "$SHELL" != "$(command -v zsh)" ]; then
  chsh -s "$(command -v zsh)"
fi

echo "Done. Restart your SSH session or run: exec zsh"
