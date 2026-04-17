FROM rust:1.95.0

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libdbus-1-dev \
    pkg-config \
    dbus \
    network-manager \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
RUN sed -i 's/"nmrs-gui"//' Cargo.toml && sed -i '/^$/d' Cargo.toml

COPY nmrs ./nmrs

RUN cargo build -p nmrs --release && cargo build -p nmrs

CMD ["/bin/bash"]