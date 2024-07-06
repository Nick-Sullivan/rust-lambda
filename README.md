
# How this was made
rust-analyzer, CodeLLDB

```bash
cargo lambda new rust_lambda --http


```

# Init

```bash
cd terraform
terraform init
```

# Run

```bash
cargo build
cargo lambda watch --only-lambda-apis
# Run debugger in VSCode
cargo lambda invoke --data-file request.json
```

# Deploy

```bash
cd ../terraform
terraform apply
```

# Other

cargo lambda build --release --output-format zip
cargo lambda deploy --binary-name rust_lambda RustLambda-Dev-Hello --iam-role arn:aws:iam::314077822992:role/RustLambda-Dev-Hello 

podman build -t rust_lambda .
podman run --rm -p 9001:9001 -e AWS_LAMBDA_FUNCTION_NAME="_" -e AWS_LAMBDA_FUNCTION_MEMORY_SIZE=4096 -e AWS_LAMBDA_FUNCTION_VERSION=1 -e AWS_LAMBDA_RUNTIME_API -e AWS_LAMBDA_RUNTIME_API="http://127.0.0.1:9000/.rt" rust_lambda