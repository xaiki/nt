# Use the official Rust image as a base
FROM docker.io/rust:1.85-slim-bookworm as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests first to leverage Docker cache
COPY Cargo.toml Cargo.lock ./
COPY crates/nt_core/Cargo.toml ./crates/nt_core/
COPY crates/nt_cli/Cargo.toml ./crates/nt_cli/
COPY crates/nt_inference/Cargo.toml ./crates/nt_inference/
COPY crates/nt_scrappers/Cargo.toml ./crates/nt_scrappers/
COPY crates/nt_storage/Cargo.toml ./crates/nt_storage/
COPY crates/nt_web/Cargo.toml ./crates/nt_web/

# Copy source code
COPY crates/nt_core/src ./crates/nt_core/src
COPY crates/nt_cli/src ./crates/nt_cli/src
COPY crates/nt_inference/src ./crates/nt_inference/src
COPY crates/nt_scrappers/src ./crates/nt_scrappers/src
COPY crates/nt_storage/src ./crates/nt_storage/src
COPY crates/nt_web/src ./crates/nt_web/src

# Build the application
ARG FEATURES
RUN cargo build --release --bin nt --features ${FEATURES}

# Create a new stage with a minimal image
FROM docker.io/debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/nt .

# Set environment variables with defaults
ENV SCRAPE_INTERVAL=3600
ENV STORAGE=sqlite
ENV MODEL_URL=""
ENV BACKEND_URL=""

# Create a shell script to handle the command
RUN echo '#!/bin/sh\n\
if [ -n "$MODEL_URL" ]; then\n\
    MODEL_ARG="--model-url $MODEL_URL"\n\
fi\n\
if [ -n "$BACKEND_URL" ]; then\n\
    BACKEND_ARG="--backend-url $BACKEND_URL"\n\
fi\n\
if [ -n "$STORAGE" ]; then\n\
    STORAGE_ARG="--storage $STORAGE"\n\
fi\n\
./nt ${STORAGE_ARG} ${MODEL_ARG} ${BACKEND_ARG} scrape source --interval ${SCRAPE_INTERVAL}' > /app/entrypoint.sh && \
    chmod +x /app/entrypoint.sh

# Set the shell script as the entrypoint
ENTRYPOINT ["/app/entrypoint.sh"] 