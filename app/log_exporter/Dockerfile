FROM rust:1.85 AS builder
WORKDIR /usr/src
COPY . .
RUN cargo install --path app/log_exporter

FROM debian:bookworm-slim
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/log_exporter /usr/local/bin/log_exporter
CMD ["log_exporter"]
