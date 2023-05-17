FROM docker.io/library/ubuntu:22.04

# show backtraces
ENV RUST_BACKTRACE 1

# install tools and dependencies
RUN apt-get update && \
	DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
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

# copy the compiled binary to the container
COPY --chown=node-template:node-template --chmod=774 node-template /usr/bin/node-template

# check if executable works in this container
RUN /usr/bin/node-template --version

# ws_port
EXPOSE 9930 9333 9944 30333 30334

VOLUME ["/node-template"]

ENTRYPOINT ["/usr/bin/node-template"]
