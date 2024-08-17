FROM rust:1.79 as builder

# Install system dependencies
RUN apt-get update \
    && apt-get install -y musl-tools musl libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Build the library, installing deps first to cache
ARG LIB_NAME=lambda_lib
RUN mkdir -p /build/${LIB_NAME}
WORKDIR /build/${LIB_NAME}
ADD ${LIB_NAME}/Cargo.toml Cargo.toml
ADD ${LIB_NAME}/Cargo.lock Cargo.lock
RUN mkdir src \
    && echo "fn main() {}" > src/main.rs \
    && rustup target add x86_64-unknown-linux-musl \
    && cargo build --release --target x86_64-unknown-linux-musl \
    && rm -f target/x86_64-unknown-linux-musl/release/deps/${LIB_NAME}* \
    && rm src/main.rs

ADD ${LIB_NAME}/src src
RUN cargo build --release --target x86_64-unknown-linux-musl \
    && strip target/x86_64-unknown-linux-musl/release/${LIB_NAME}

# Build the cloud entrypoint, installing deps first to cache
ARG ENTRY_NAME=lambda_entry
WORKDIR /build/${ENTRY_NAME}
ADD ${ENTRY_NAME}/Cargo.toml Cargo.toml
ADD ${ENTRY_NAME}/Cargo.lock Cargo.lock
ADD ${ENTRY_NAME}/src src
RUN cargo build --release --target x86_64-unknown-linux-musl \
    && strip target/x86_64-unknown-linux-musl/release/${ENTRY_NAME}

# AWS Lambda Image
FROM public.ecr.aws/lambda/provided:al2

ARG ENTRY_NAME=lambda_entry

COPY --from=builder /build/${ENTRY_NAME}/target/x86_64-unknown-linux-musl/release/${ENTRY_NAME} /entry

ENTRYPOINT ["/entry"]