FROM rust:1.79 as builder

WORKDIR /build

# Install system dependencies
RUN apt-get update \
    && apt-get install -y musl-tools musl libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Install project dependencies, in a seperate step to cache them
ADD Cargo.toml Cargo.toml
ADD Cargo.lock Cargo.lock
RUN mkdir src \
    && echo "fn main() {}" > src/main.rs \
    && rustup target add x86_64-unknown-linux-musl \
    && cargo build --release --target x86_64-unknown-linux-musl \
    && rm -f target/x86_64-unknown-linux-musl/release/deps/rust_lambda* \
    && rm src/main.rs

# Build the project
ADD src src
RUN cargo build --release --target x86_64-unknown-linux-musl \
    && strip /build/target/x86_64-unknown-linux-musl/release/rust_lambda

# AWS Lambda Image
FROM public.ecr.aws/lambda/provided:al2

COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/rust_lambda /rust_lambda

ENTRYPOINT ["/rust_lambda"]