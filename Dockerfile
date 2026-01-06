FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    network-manager \
    dbus \
    build-essential \
    curl \
    pkg-config \
    libdbus-1-dev \
    && rm -rf /var/lib/apt/lists/*

# Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# DBus runtime dirs
RUN mkdir -p /run/dbus /run/NetworkManager

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY nmrs ./nmrs

ENV CARGO_NET_OFFLINE=false

CMD ["sh", "-c", "dbus-daemon --system --fork && sleep 1 && NetworkManager --no-daemon & sleep 3 && cargo test -p nmrs"]

