# Original Containerfile:
# https://github.com/paritytech/polkadot/blob/9428d8c3ebcd4ec5fb9095549be7d5f84c48d8a2/scripts/ci/dockerfiles/polkadot_injected_release.Dockerfile

FROM docker.io/library/ubuntu:20.04

# show backtraces
ENV RUST_BACKTRACE 1

# install tools and dependencies
RUN apt-get update && \
	DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
		libssl1.1 \
		ca-certificates && \
  useradd -m -u 1000 -U -s /bin/sh -d /node-template node-template && \
# apt cleanup
	apt-get autoremove -y && \
	apt-get clean && \
	rm -rf /var/lib/apt/lists/* && \
	mkdir -p /data /node-template/.local/share && \
	chown -R node-template:node-template /data && \
	ln -s /data /node-template/.local/share/node-template

USER node-template

# check if executable works in this container
RUN /usr/bin/node-template --version

COPY node-template /usr/bin/node-template

# ws_port
EXPOSE 9930 9333 9944 30333 30334

VOLUME ["/node-template"]

ENTRYPOINT ["/usr/bin/node-template"]
