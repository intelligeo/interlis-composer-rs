ARG ILI2C_VERSION=5.6.9

FROM rust:1.90-bookworm AS builder
WORKDIR /app

COPY Cargo.toml ./
COPY src ./src
COPY static ./static

RUN cargo build --release

FROM debian:bookworm-slim
ARG ILI2C_VERSION
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl unzip openjdk-17-jre-headless \
    && mkdir -p /tools \
    && curl -fsSL "https://downloads.interlis.ch/ili2c/ili2c-${ILI2C_VERSION}.zip" -o /tmp/ili2c.zip \
    && unzip -q /tmp/ili2c.zip -d /tmp/ili2c \
    && ILI2C_JAR="$(find /tmp/ili2c -type f -name 'ili2c*.jar' | head -n 1)" \
    && test -n "$ILI2C_JAR" \
    && cp "$ILI2C_JAR" /tools/ili2c.jar \
    && rm -rf /tmp/ili2c /tmp/ili2c.zip \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/interlis-composer-rs /app/interlis-composer-rs
COPY --from=builder /app/static /app/static

ENV PORT=8080
ENV ILI2C_ENABLED=true
ENV ILI2C_BIN=java
ENV ILI2C_ARGS="-jar /tools/ili2c.jar"
EXPOSE 8080

CMD ["/app/interlis-composer-rs"]
