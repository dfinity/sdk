#! /usr/bin/env sh

set -u

# Install toml with `cargo install toml-cli`.

 tee Cargo.toml <<EOF
$(toml set Cargo.toml package.version $1)
EOF
