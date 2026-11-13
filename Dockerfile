FROM rust:1.100.0-slim-trixie@sha256:e9d95b74296f7126a83ae3d041aeaba1ff4c478f13abc5cc034fcf7c131c3d4a AS builder
RUN apt-get update && apt-get install -y \
  pkg-config \
  libssl-dev && \
  rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
ENV SKIP_ASSET_BUILD=1
RUN cargo build --release --package hakanai-server

FROM gcr.io/distroless/cc-debian13@sha256:809d04afa1be033c441f76bb0051878c342fb76d0ea236bc6dc94b655fa0fbd6
COPY --from=builder /src/target/release/hakanai-server /app/hakanai-server
ADD ./server/custom /custom
ENV HAKANAI_CUSTOM_ASSETS_DIR=/custom
USER nonroot
EXPOSE 8080
ENTRYPOINT ["/app/hakanai-server"]
