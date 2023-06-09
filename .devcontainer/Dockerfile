# Dockerfile
# This Dockerfile is used to build an image that includes the Docker CLI tool in addition to the software present
# in the universal base image. This can be useful when you need to run Docker commands within the Docker container.

# Start from the universal base image
FROM mcr.microsoft.com/devcontainers/universal:2

# Install Docker CLI
RUN apt-get update && \
    apt-get install -y apt-transport-https ca-certificates curl software-properties-common gnupg lsb-release && \
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | apt-key add - && \
    echo "deb [arch=amd64] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null && \
    apt-get update && \
    apt-get install -y --no-install-recommends docker-ce-cli protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*
