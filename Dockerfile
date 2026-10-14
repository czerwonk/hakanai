FROM rust:1.99.0-slim-trixie@sha256:2752b332db73fdbb7dc576f06c82ed1f312005784ef913d7e04a28f5f55dc581 AS builder
RUN apt-get update && apt-get install -y \
  pkg-config \
  libssl-dev && \
  rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
ENV SKIP_ASSET_BUILD=1
RUN cargo build --release --package hakanai-server

FROM gcr.io/distroless/cc-debian13
COPY --from=builder /src/target/release/hakanai-server /app/hakanai-server
ADD ./server/custom /custom
ENV HAKANAI_CUSTOM_ASSETS_DIR=/custom
USER nonroot
EXPOSE 8080
ENTRYPOINT ["/app/hakanai-server"]
