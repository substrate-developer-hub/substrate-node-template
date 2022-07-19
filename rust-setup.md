---
title: Installation
---

This page will guide you through the **2 steps** needed to prepare a computer for **Substrate** development.
Since Substrate is built with [the Rust programming language](https://www.rust-lang.org/), the first
thing you will need to do is prepare the computer for Rust development - these steps will vary based
on the computer's operating system. Once Rust is configured, you will use its toolchains to interact
with Rust projects; the commands for Rust's toolchains will be the same for all supported,
Unix-based operating systems.

## 1. Build dependencies

Substrate development is easiest on Unix-based operating systems like macOS or Linux. The examples
in the Substrate [Tutorials](/tutorials) and [How-to Guides](/how-to-guides)
use Unix-style terminals to demonstrate how to interact with Substrate from the command line.

### Ubuntu/Debian

Use a terminal shell to execute the following commands:

```bash
sudo apt update
# May prompt for location information
sudo apt install -y git clang curl libssl-dev llvm libudev-dev pkg-config make
```

### Arch Linux

Run these commands from a terminal:

```bash
pacman -Syu --needed --noconfirm curl git clang make
```

### Fedora

Run these commands from a terminal:

```bash
sudo dnf update
sudo dnf install clang curl git openssl-devel make
```

### OpenSUSE

Run these commands from a terminal:

```bash
sudo zypper install clang curl git openssl-devel llvm-devel libudev-devel make
```

### macOS

> If you have an Apple M1 ARM system on a chip, make sure that you have Apple Rosetta 2 installed through \`softwareupdate --install-rosetta\`. This is only needed to run the \`protoc\` tool during the build. The build itself and the target binaries would remain native."

Open the Terminal application and execute the following commands:

```bash
# Install Homebrew if necessary https://brew.sh/
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)"

# Make sure Homebrew is up-to-date, install openssl
brew update
brew install openssl
```

### Windows

> Native development of Substrate is _not_ very well supported!
> It is _highly_ recommend to
use [Windows Subsystem Linux](https://docs.microsoft.com/en-us/windows/wsl/install-win10) (WSL)
and follow the instructions for [Ubuntu/Debian](#ubuntudebian).

## 2. Rust developer environment

This guide uses <https://rustup.rs> installer and the `rustup` tool to manage the Rust toolchain.
First install and configure `rustup`:

```bash
# Install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Configure
source ~/.cargo/env
```

Configure the Rust toolchain to default to the latest stable version, add nightly and the nightly wasm target:

```bash
rustup default stable
rustup update
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

## Test your set-up

Now the best way to ensure that you have successfully prepared a computer for Substrate development is to follow the platform-specific steps in [Install](https://docs.substrate.io/main-docs/install/).

If you have trouble compiling the [Substrate node template](https://github.com/substrate-developer-hub/substrate-node-template), see [Troubleshoot Rust issues](https://docs.substrate.io/main-docs/install/troubleshooting/).