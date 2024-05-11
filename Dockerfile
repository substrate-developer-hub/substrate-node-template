# Use Parity's official CI image as the builder
FROM docker.io/library/ubuntu:20.04 as builder
WORKDIR /plenitud
COPY . /plenitud

# Install dependencies in one RUN to reduce layer size and avoid cache issues
RUN apt-get update && apt-get install -y \
    git \
    build-essential \
    clang \
    curl \
    libssl-dev \
    llvm \
    libudev-dev \
    make \
    protobuf-compiler && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    . $HOME/.cargo/env && \
    rustup target add wasm32-unknown-unknown --toolchain nightly && \
    cargo build --release

# Start from a minimal Ubuntu image for the runtime environment
FROM docker.io/library/ubuntu:20.04
LABEL description="Plenitud Node"

# Copy the build artifact from the builder stage
COPY --from=builder /plenitud/target/release/plenitud /usr/local/bin
COPY --from=builder /plenitud/init-node-server.sh /usr/local/bin
COPY --from=builder /plenitud/spec.json /usr/local/bin

# Make scripts executable
RUN chmod +x /usr/local/bin/init-node-server.sh /usr/local/bin/plenitud

# Setup user and directories
RUN useradd -m -u 1000 -U -s /bin/bash -d /node-dev node-dev && \
    mkdir -p /chain-data /node-dev/.local/share /data/node01 && \
    chown -R node-dev:node-dev /chain-data /data/node01 && \
    ln -s /chain-data /node-dev/.local/share/academy-pow

# Switch to user
USER node-dev

# Expose necessary ports
EXPOSE 30333 9933 9944 9615

# Define volume for chain data
VOLUME ["/chain-data"]

# Set the container's main command
CMD ["/usr/local/bin/init-node-server.sh"]
