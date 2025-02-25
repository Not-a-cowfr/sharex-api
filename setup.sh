#!/bin/bash
set -e

echo "Starting setup..."
echo "Checking Rust installation..."

if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "Rust already installed, updating..."
    rustup update
fi

echo "Verifying Rust installation..."
rustc --version
cargo --version

echo "Building..."
cargo build --release

echo "Done!"