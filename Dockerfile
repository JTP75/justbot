FROM rust:latest as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY bin ./bin

RUN cargo build --release --bin puetce-server --bin puetce-client

FROM debian:trixie-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl3 nodejs npm \
    curl \
    poppler-utils \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/puetce-server ./

EXPOSE 8081

CMD ["./puetce-server"]