#!/usr/bin/env bash
set -euo pipefail

# Run inside Ubuntu (WSL). Uses the official Rust installer.
apt-get update -qq
DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends build-essential ca-certificates curl

if [ ! -x "$HOME/.cargo/bin/rustup" ]; then
  curl --proto '=https' --tlsv1.2 --fail --silent --show-error --max-time 60 \
    https://sh.rustup.rs -o /tmp/reserve-rustup.sh
  sh /tmp/reserve-rustup.sh -y --profile minimal --default-toolchain stable
fi

. "$HOME/.cargo/env"
rustup component add rustfmt clippy
rustc --version
cargo --version
