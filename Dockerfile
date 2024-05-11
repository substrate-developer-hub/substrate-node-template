# Use Debian Bullseye for the builder to ensure compatibility with newer libraries
FROM debian:bullseye as builder
WORKDIR /plenitud
COPY . /plenitud

# Install dependencies and build the project
RUN apt-get update && apt-get install -y \
    git build-essential cmake clang curl libssl-dev llvm libudev-dev make protobuf-compiler \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && . $HOME/.cargo/env \
    && rustup default nightly \
    && rustup update \
    && cargo build --release

# Use the same Debian Bullseye image for runtime to avoid library mismatches
FROM debian:bullseye
LABEL description="Plenitud Node"

# Copy the build artifact from the builder stage
COPY --from=builder /plenitud/target/release/node-template /usr/local/bin
COPY --from=builder /plenitud/init-node-server.sh /usr/local/bin
COPY --from=builder /plenitud/spec.json /usr/local/bin

# Make scripts executable
RUN chmod +x /usr/local/bin/init-node-server.sh /usr/local/bin/node-template

# Setup user and directories
RUN useradd -m -u 1000 -U -s /bin/bash -d /node-dev node-dev && \
    mkdir -p /chain-data /node-dev/.local/share /data/node01 && \
    chown -R node-dev:node-dev /chain-data /data/node01 && \
    ln -s /chain-data /node-dev/.local/share/node-template

# Switch to user
USER node-dev

# Expose necessary ports
EXPOSE 30333 9933 9944 9615

# Define volume for chain data
VOLUME ["/chain-data"]

# Set the container's main command
CMD ["/usr/local/bin/init-node-server.sh"]
