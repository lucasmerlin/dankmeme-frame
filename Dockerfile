FROM rust:1.65 as builder
WORKDIR /usr/src/myapp
COPY . .
RUN cargo install --path ./dank-server


# Runtime image
FROM debian:bullseye-slim

RUN apt update && apt install -y libssl-dev

# Run as "app" user
RUN useradd -ms /bin/bash app

USER app
WORKDIR /app


# Get compiled binaries from builder's cargo install directory
COPY --from=builder /usr/local/cargo/bin/dank-server /app/dank-server

# No CMD or ENTRYPOINT, see fly.toml with `cmd` override.