FROM rust:1.99.0-slim-trixie@sha256:a32456165ecc2347c799bba7b54f5aabc158831934b2b98932b81080b312de62 AS builder
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
