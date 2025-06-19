# Use official Ubuntu base image
FROM ubuntu:latest

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

# Install specific Rust nightly version
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain nightly -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install additional Rust components
RUN rustup component add rust-src llvm-tools-preview
RUN cargo install cargo-fuzz

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
