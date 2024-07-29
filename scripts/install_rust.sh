#!/bin/sh

echo "*** Checking if Rust is already installed"

if which rustup >/dev/null 2>&1; then
    echo "Rust is already installed. Exiting."
    exit 0
fi

echo "*** Installing Rust"

if [[ "$(uname)" == "Darwin" ]]; then
    # macOS
    if ! which brew >/dev/null 2>&1; then
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)"
    fi
    
    brew update
    brew install openssl cmake llvm
    elif [[ "$(uname)" == "Linux" ]]; then
    if [[ -f "/etc/arch-release" ]]; then
        # Arch Linux
        sudo pacman -Syu --noconfirm
        sudo pacman -S --noconfirm cmake pkgconf openssl git gcc clang
    else
        # Ubuntu (and other Debian-based distributions)
        sudo apt-get update
        sudo apt-get install -y cmake pkg-config libssl-dev git gcc build-essential clang libclang-dev
    fi
else
    echo "Unsupported operating system. Exiting."
    exit 1
fi

curl https://sh.rustup.rs -sSf | sh -s -- -y
source "$HOME/.cargo/env"
rustup default stable

rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly

echo "*** Rust installation complete"
