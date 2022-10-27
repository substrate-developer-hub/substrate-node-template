# This is the build stage for Polkadot. Here we create the binary in a temporary image.
FROM docker.io/paritytech/ci-linux:production as builder

WORKDIR /node-template
COPY . .
RUN cargo build --locked --release

# This is the 2nd stage: a very small image where we copy the Polkadot binary."
FROM docker.io/library/ubuntu:20.04
LABEL description="Multistage Docker image for NODE_TEMPLATE" \
  image.type="builder" \
  image.authors="support@onfinality.io" \
  image.vendor="OnFinality" \
  image.description="Multistage Docker image for NODE_TEMPLATE" \
  image.source="https://github.com/OnFinality-io/substrate-node-template" \
  image.documentation="https://github.com/OnFinality-io/substrate-node-template"

# Copy the node binary.
COPY --from=builder /node-template/target/release/node-template /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /node-dev node-dev && \
  mkdir -p /chain-data /node-dev/.local/share && \
  chown -R node-dev:node-dev /chain-data && \
  ln -s /chain-data /node-dev/.local/share/node-template && \
  # unclutter and minimize the attack surface
  rm -rf /usr/bin /usr/sbin && \
  # check if executable works in this container
  /usr/local/bin/node-template --version

USER node-dev

EXPOSE 30333 9933 9944 9615
VOLUME ["/chain-data"]

ENTRYPOINT ["/usr/local/bin/node-template"]