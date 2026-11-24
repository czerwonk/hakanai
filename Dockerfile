FROM rust:1.100.0-slim-trixie@sha256:6f14b2b4b278ec0d50aa1d69541c304ce0f95e0f6268487aa9924af1c2e9aebe AS builder
RUN apt-get update && apt-get install -y \
  pkg-config \
  libssl-dev && \
  rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
ENV SKIP_ASSET_BUILD=1
RUN cargo build --release --package hakanai-server

FROM gcr.io/distroless/cc-debian13@sha256:dac90a25ffe03ebaace90cda886b081e61ab6cd77ac47eda51f7bcc15c320765
COPY --from=builder /src/target/release/hakanai-server /app/hakanai-server
ADD ./server/custom /custom
ENV HAKANAI_CUSTOM_ASSETS_DIR=/custom
USER nonroot
EXPOSE 8080
ENTRYPOINT ["/app/hakanai-server"]
