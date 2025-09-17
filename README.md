# localsetup – Setup your local computer / user automatically

[![CI](https://github.com/zgtm/localsetup/actions/workflows/ci.yaml/badge.svg)](https://github.com/zgtm/localsetup/actions/workflows/ci.yaml)

## Installation

You can find [the latest release on Github](https://github.com/zgtm/localsetup/releases/latest).

Install the binary from Github directly
```
curl --create-dirs -o ~/.local/bin/localsetup https://github.com/zgtm/localsetup/releases/latest/download/localsetup  && chmod a+x ~/.local/bin/localsetup
```
or if `curl` is not installed yet
```
wget -P ~/.local/bin/localsetup  https://github.com/zgtm/localsetup/releases/latest/download/localsetup  && chmod a+x ~/.local/bin/localsetup
```

or build and install it with cargo:
```
cargo install localsetup
```
(or checkout this repository and run `cargo install --path .` inside).

## First Setup

Create a setupfile (see below) somewhere and pass it to localsetup:

```
localsetup <path or URL to setupfile>
```

Though, if you don't have ~/.local/bin in `$PATH` yet, you might need to run

```
~/.local/bin/localsetup <path or URL to setupfile>
```

## Subsequent Setup

Just run

```
localsetup
```

though, if you still don't have ~/.local/bin in `$PATH`, use this instead:

```
~/.local/bin/localsetup
```

## The Setupfile

Currently supported:

### Setup SSH

```
[ssh]
###
no_passphrase = false
```
### Setup Git

```
[git]
name = "name" 
email = "name@example.com"
```

### Install Rust via rustup
```
[rustup]
install_rust = false
update_rust = false
```

### Remove Snap on Ubuntu and install Firefox from PPA
``` 
[ubuntu]
remove_snap_and_install_firefox_ppa = false
remove_snap_and_install_firefox_ppa_yes_delete_my_bookmarks_and_everything = false
```

### Install ghostty
```
[ghostty]
install_ghosty_from_ghostty_ubuntu = true
```

### Install uv
``` 
[uv]
install_astral_sh = true
```

### Setup XDG user directories
```
[xdg-user-dirs]
move_existing = true
desktop = "$HOME/Desktop"
documents = "$HOME/Documents"
downloads = "$HOME/Downloads"
music = "$HOME/Music"
pictures = "$HOME/Pictures"
publicshare = "$HOME/Public"
templates = "$HOME/Templates"
videos = "$HOME/Videos"
```

### Install or remove packages
```
[packages]
install = []
install_list = fi
```

### Checkout git repositories
```
[[repositories]]
source = "git@github.com:example/dotfiles.git"
target = "~/dotfiles"
update = false
synchronise = false
run_always = "~/install.sh" # command to run everytime localsetup is run
```


