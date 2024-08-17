FROM rust:1.79 as builder
WORKDIR app

# Install system dependencies
RUN apt-get update \
    && apt-get install -y musl-tools musl libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/* \
    && rustup target add x86_64-unknown-linux-musl

# Build the project
COPY . .
RUN cargo build \
        --release \
        --bin api_cloud_entry \
        --target x86_64-unknown-linux-musl \
    && strip target/x86_64-unknown-linux-musl/release/api_cloud_entry
    
# Create an AWS Lambda compatible image
FROM public.ecr.aws/lambda/provided:al2
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/api_cloud_entry /entry
ENTRYPOINT ["/entry"]
