FROM rust:slim-bookworm as builder
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    npm \
    lld \
    curl && \
    rm -rf /var/lib/apt/lists/*
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
RUN rustup target add wasm32-unknown-unknown
WORKDIR /src
COPY . .
RUN npm install && cd server && cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /src/target/release/hakanai-server /app/hakanai-server
USER nonroot
EXPOSE 8080
ENTRYPOINT ["/app/hakanai-server"]
