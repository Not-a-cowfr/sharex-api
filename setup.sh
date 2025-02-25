#!/bin/bash

set -e

echo "Starting setup..."

echo "Updating system package manager..."
sudo apt update -y

echo "Installing system dependencies..."
sudo apt install -y \
    build-essential \
    curl \
    libssl-dev \
    pkg-config

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

echo "Building and running the sharex-uploader app..."
cargo build --release
echo "Build complete! Starting the app..."
cargo run --release

echo "Done!"