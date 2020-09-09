# Substrate Developer Set-Up Guide

The purpose of this document is to provide developers with the information they need to set-up a Substrate development
environment. For each operating system we support a standard Docker container exists to verify that the
provided steps are sufficient to build and run the node template in this repository.

## Ubuntu

This set-up was verified using [the standard Ubuntu Docker container](https://hub.docker.com/_/ubuntu). This container
defaults to a root user, so most developers should preface the following commands with `sudo` in order
to execute them with the correctly defined privileges.

```shell
apt update
# May prompt for location information
apt install -y cmake pkg-config libssl-dev git build-essential clang libclang-dev curl
# Select 1 to proceed with standard installation
curl https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env
rustup default stable
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
rustup update
```

## Arch Linux

This set-up was verified using [the standard Arch Linux Docker container](https://hub.docker.com/_/archlinux). This
container defaults to a root user, so most developers should preface the following commands with `sudo`
in order to execute them with the correctly defined privileges.

```shell
pacman -Syu --needed --noconfirm cmake gcc openssl-1.0 pkgconf git clang
export OPENSSL_LIB_DIR="/usr/lib/openssl-1.0"
export OPENSSL_INCLUDE_DIR="/usr/include/openssl-1.0"
# Select 1 to proceed with standard installation
curl https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env
rustup default stable
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
rustup update
```

## Unsupported Operating Systems

Any operating system not explicitly described by this document should be considered unsupported for Substrate
development.
