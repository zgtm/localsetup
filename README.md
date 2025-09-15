# localsetup – Setup your local computer / user automatically

[![CI](https://github.com/zgtm/localsetup/actions/workflows/ci.yaml/badge.svg)](https://github.com/zgtm/localsetup/actions/workflows/ci.yaml)

## Installation

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

 - `setup_ssh_key` (boolean, default `true`) Setup an ssh key and print the public key if none exists yet
