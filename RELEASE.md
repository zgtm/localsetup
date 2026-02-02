```
export NEW_VERSION="0.0.4"
sed -i 's/^version = ".*"$/version = "'$NEW_VERSION'"/' Cargo.toml
cargo run
git commit -am "Change version to $NEW_VERSION"
git tag -a v$NEW_VERSION -m "Version $NEW_VERSION"
cargo publish
```
