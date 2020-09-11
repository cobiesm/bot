#!/bin/bash
cargo clippy -Z unstable-options || exit 1
PKG_CONFIG_ALLOW_CROSS=1 cargo build --release --target x86_64-unknown-linux-musl \
    || exit 1
cp target/x86_64-unknown-linux-musl/release/hello_worlds-bot deploy/bin/ || exit 1

cd deploy
git add --all && git commit -m "a" && git push --force heroku master
