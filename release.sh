#!/bin/bash

if [ -z "$1" ]; then
    echo "No version argument supplied"
    exit
fi

sed -i 's/^version = ".*"$/version = "'$1'"/' Cargo.toml
cargo run
cargo publish
git commit -am "Change version to $1"
git tag -a v$1 -m "Version $1"
git push
