#!/bin/bash

set -e

echo "Starting setup for sharex-uploader Rust app..."

echo "Updating system package manager..."
if command -v sudo &> /dev/null; then
    sudo apt update -y || { echo "Failed to update package manager with sudo"; exit 1; }
else
    apt update -y || { echo "Failed to update package manager"; exit 1; }
fi

echo "Installing system dependencies..."
if command -v sudo &> /dev/null; then
    sudo apt install -y \
        build-essential \
        curl \
        libssl-dev \
        pkg-config || { echo "Failed to install dependencies with sudo"; exit 1; }
else
    apt install -y \
        build-essential \
        curl \
        libssl-dev \
        pkg-config || { echo "Failed to install dependencies"; exit 1; }
fi

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
cargo build --release || { echo "Build failed"; exit 1; }
echo "Build complete! Starting the app..."
cargo run --release || { echo "Run failed"; exit 1; }

echo "Setup and run complete! Your app should be running now."