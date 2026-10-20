FROM rust:1.99.1-slim AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY tests ./tests
RUN cargo test --locked
RUN cargo build --release --locked

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/csvdelta /usr/local/bin/csvdelta
ENTRYPOINT ["csvdelta"]
