FROM rust:slim-bookworm as builder
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
ENV SKIP_ASSET_BUILD=1
RUN cargo build --release --package hakanai-server

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /src/target/release/hakanai-server /app/hakanai-server
ADD ./server/custom /custom
ENV HAKANAI_CUSTOM_ASSETS_DIR=/custom
USER nonroot
EXPOSE 8080
ENTRYPOINT ["/app/hakanai-server"]
