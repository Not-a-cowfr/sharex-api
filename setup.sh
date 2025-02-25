#!/bin/bash

set -e

echo "Starting setup for Rust app environment..."

echo "Updating system package manager..."
sudo apt update -y

echo "Installing system dependencies..."
sudo apt install -y build-essential curl libssl-dev pkg-config

echo "Installing Rust..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

source "$HOME/.cargo/env"

echo "Verifying Rust installation..."
rustc --version
cargo --version

echo "Setup complete! You can now build and run your Rust app."
echo "Try 'cargo build' or 'cargo run' in your project directory."