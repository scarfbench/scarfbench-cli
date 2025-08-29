FROM ubuntu:22.04

# Prevent interactive prompts
ENV DEBIAN_FRONTEND=noninteractive

# Install Java 17, Maven, Git, Curl, and npm
RUN apt-get update && \
    apt-get install -y openjdk-17-jdk maven git curl npm && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# Install the Cursor CLI and add its directory to the PATH
RUN curl https://cursor.com/install -fsS | bash && \
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
