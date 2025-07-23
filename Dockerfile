FROM rust:slim-bookworm as builder
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
ENV SKIP_TYPESCRIPT_BUILD=1
ENV SKIP_WASM_BUILD=1
RUN cargo build --release --package hakanai-server

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /src/target/release/hakanai-server /app/hakanai-server
USER nonroot
EXPOSE 8080
ENTRYPOINT ["/app/hakanai-server"]
