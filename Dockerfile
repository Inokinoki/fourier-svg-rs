# Dockerfile for building Fourier SVG Tauri App on Ubuntu 24.04
# This container builds the Tauri desktop application with all dependencies

FROM ubuntu:24.04

# Prevent interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive
ENV DISPLAY=:0

# Install build dependencies for Tauri app on Linux
RUN apt-get update && apt-get install -y \
    # Build essentials
    build-essential \
    curl \
    pkg-config \
    libssl-dev \
    # WebKitGTK for Tauri
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    # Additional useful tools
    git \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Set working directory
WORKDIR /app

# Copy Cargo files first for better Docker layer caching
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY fourier-svg ./fourier-svg
COPY fourier-cli ./fourier-cli
COPY gpui-app ./gpui-app
COPY tauri-app ./tauri-app

# Build the Tauri app
RUN cargo build --release --features tauri -p tauri-app

# Set the entrypoint
ENTRYPOINT ["/root/.cargo/bin/cargo"]
CMD ["run", "--release", "--features", "tauri", "-p", "tauri-app"]
