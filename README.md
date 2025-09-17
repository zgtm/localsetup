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

The following settings are currently supported:

### Setup SSH

An SSH-Key will be created by default, if it does not exist.

```
[ssh]
###
no_passphrase = false
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
Can install `ghostty` from the [ghostty-ubuntu](https://github.com/mkasberg/ghostty-ubuntu) repository.
```
[ghostty]
install_ghosty_from_ghostty_ubuntu = true
```

### Install uv
Can install `uv` from the astral.sh website.
``` 
[uv]
install_astral_sh = true
```

### Setup XDG user directories
This will update `user-dirs.dirs` accordingly. If wanted, it can try to move existing directories to the new location.
```
[xdg-user-dirs]
move_existing = false
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
install_list = "packages_to_install.txt"
remove = []
install_list = "packages_to_remove.txt"
```

### Setup Git

Setup name and email so git does not complain when checking out repositories afterwards.

```
[git]
name = "name" 
email = "name@example.com"
```

### Checkout git repositories

This will checkout git repositories. If wanted, it can update (`git pull`) or synchronize (`git pull && git push`) whenever localsetup is run. Also can run arbitrary commands.

```
[[repositories]]
source = "git@github.com:example/dotfiles.git"
target = "~/dotfiles"
update = false
synchronise = false
run_always = "~/install.sh" # command to run everytime localsetup is run
```


