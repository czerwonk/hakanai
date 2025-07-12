FROM rust:slim-bookworm as builder
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    npm && \
    npm install -g typescript && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
RUN cd server && cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /src/target/release/hakanai-server /app/hakanai-server
USER nonroot
EXPOSE 8080
ENTRYPOINT ["/app/hakanai-server"]
