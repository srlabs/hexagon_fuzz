# Use official Ubuntu base image
FROM ubuntu:24.04

# Update package lists and install required dependencies
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y \
    curl \
    python3 \
    python3-pip \
    python3-sphinx \
    python3-sphinx-rtd-theme \
    ninja-build \
    libglib2.0-dev \
    flex \
    bison \
    clang \
    git \
    vim \
    tmux \
    gdb \
    gdbserver \
    socat \
    && rm -rf /var/lib/apt/lists/*

# Setup ncurses5, required for hexagon sdk 5.5.5.0, and other dependencies for custom tmux tooling
RUN echo "deb http://security.ubuntu.com/ubuntu focal-security main universe" > /etc/apt/sources.list.d/ubuntu-focal-sources.list
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y \
    python-is-python3\
    libncurses5 \
    lsb-release \
    && rm -rf /var/lib/apt/lists/*


# Install specific Rust nightly version
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install additional Rust components
RUN rustup component add rust-src llvm-tools-preview

# Create and set fuzz directory
WORKDIR /fuzz

# Copy required files and directories
COPY Cargo.toml .
COPY .cargo ./.cargo
COPY firmware_config.json .
COPY qdsp6sw.mbn .
COPY qemu-libafl-bridge ./qemu-libafl-bridge
COPY rust-toolchain.toml .
COPY scripts ./scripts
COPY src ./src
COPY corpus ./corpus

# Set the environment variables
ENV LLVM_CONFIG_PATH=/usr/bin/llvm-config-18
ENV SDK_HOME=.

# Build
# RUN cargo build --release
