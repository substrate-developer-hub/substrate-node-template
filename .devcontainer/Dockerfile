# Start from the universal base image
FROM mcr.microsoft.com/devcontainers/universal:2

# Install Docker CLI
RUN apt-get update && \
    apt-get install -y apt-transport-https ca-certificates curl software-properties-common gnupg lsb-release && \
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | apt-key add - && \
    echo "deb [arch=amd64] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null && \
    apt-get update && \
    apt-get install -y --no-install-recommends docker-ce-cli && \
    rm -rf /var/lib/apt/lists/*
