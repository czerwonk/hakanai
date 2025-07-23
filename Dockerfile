FROM rust:slim-bookworm as builder
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    npm \
    lld \
    curl && \
    rm -rf /var/lib/apt/lists/*

# Install wasm-pack from pre-built binary (much faster than cargo install)
RUN curl -sSL https://github.com/rustwasm/wasm-pack/releases/download/v0.13.1/wasm-pack-v0.13.1-x86_64-unknown-linux-musl.tar.gz \
    | tar -xzf - -C /usr/local/bin --strip-components=1 wasm-pack-v0.13.1-x86_64-unknown-linux-musl/wasm-pack

# Add wasm32 target for Rust
RUN rustup target add wasm32-unknown-unknown
WORKDIR /src
COPY . .
RUN npm install && cd server && cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /src/target/release/hakanai-server /app/hakanai-server
USER nonroot
EXPOSE 8080
ENTRYPOINT ["/app/hakanai-server"]
