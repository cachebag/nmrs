FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    network-manager \
    dbus \
    build-essential \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN mkdir -p /run/dbus

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY nmrs ./nmrs
COPY nmrs-gui ./nmrs-gui

RUN mkdir -p /run/NetworkManager

CMD ["sh", "-c", "dbus-daemon --system --fork && sleep 1 && NetworkManager --no-daemon &  sleep 3 && cargo test -p nmrs"]
