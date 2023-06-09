# build.Dockerfile
# This Dockerfile is used to build an image containing the Substrate binary.
# The primary use for this binary is to setup GitHub Codespaces.
# To build the image, navigate to the directory containing this file and run:
# docker build -t your-image-name -f build.Dockerfile .

# Using a base image
FROM debian:stable-slim

# Working directory
WORKDIR /usr/local/bin

# Copy your binary
COPY ./target/debug/node-template .

RUN chmod +x node-template && \
# Install dependencies
    apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 && \
# apt clean up
    apt-get autoremove -y && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*


# After building the Docker image, you can push it to Docker Hub by following these steps:
# 1. Log in to Docker Hub from your command line using: docker login --username=your-username
# 2. Tag the image with your Docker Hub username: docker tag your-image-name your-username/your-image-name
# 3. Push the image to Docker Hub: docker push your-username/your-image-name
