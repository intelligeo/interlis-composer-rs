FROM rust:1.90-bookworm AS builder
WORKDIR /app

COPY Cargo.toml ./
COPY src ./src
COPY static ./static

RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/interlis-composer-rs /app/interlis-composer-rs
COPY --from=builder /app/static /app/static

ENV PORT=8080
EXPOSE 8080

CMD ["/app/interlis-composer-rs"]
