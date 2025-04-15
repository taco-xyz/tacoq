# Base image for chef ---------------------------------------------------------

FROM rust:1.86.0 AS chef

# =============================================================================
# Architecture-specific build:
#   - aarch64-unknown-linux-musl
#   - x86_64-unknown-linux-musl
# =============================================================================

# Verify ARCH is set to a valid architecture
ARG ARCH
RUN if [ "$ARCH" != "aarch64-unknown-linux-musl" ] && [ "$ARCH" != "x86_64-unknown-linux-musl" ]; then \
      echo "Error: ARCH must be either 'aarch64-unknown-linux-musl' or 'x86_64-unknown-linux-musl', but was instead '$ARCH'" && \
      exit 1; \
    fi

# Check if building on compatible architecture
RUN host_arch=$(uname -m); \
    if [ "$ARCH" = "aarch64-unknown-linux-musl" ] && [ "$host_arch" != "aarch64" ]; then \
      echo "Error: Cannot build aarch64 target on $host_arch architecture" && exit 1; \
    elif [ "$ARCH" = "x86_64-unknown-linux-musl" ] && [ "$host_arch" != "x86_64" ]; then \
      echo "Error: Cannot build x86_64 target on $host_arch architecture" && exit 1; \
    fi

# Install cargo-chef for caching compilation
WORKDIR /server
RUN cargo install cargo-chef

# Planner stage ---------------------------------------------------------------

FROM chef AS planner
WORKDIR /app

# Copy workspace files first
COPY . .

# Create recipe
RUN cargo chef prepare --recipe-path recipe-${ARCH}.json

# Builder stage ----------------------------------------------------------------
FROM chef AS builder
WORKDIR /app

# Install:
# - curl for health checks
# - musl-tools and musl-dev for building a static binary
RUN apt-get update && \
    apt-get install -y \
        curl \
        musl-tools \
        musl-dev && \
    rm -rf /var/lib/apt/lists/*

# Add the statically linked Rust target for the target architecture
RUN rustup target add ${ARCH}

# Get the Chef Recipe to build the dependencies
COPY --from=planner /app/recipe-${ARCH}.json recipe-${ARCH}.json

# Cook dependencies
RUN cargo chef cook --release --recipe-path recipe-${ARCH}.json --target ${ARCH}

COPY . .

# Build the service WITHOUT a Postgres connection
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin relay --target ${ARCH}

# Strip the binary to reduce size
RUN strip target/${ARCH}/release/relay

# Release stage ----------------------------------------------------------------

FROM scratch AS release
ARG ARCH
WORKDIR /app

# Copy the binary from the correct location
COPY --from=builder /app/target/${ARCH}/release/relay .

# Expose Axum port
EXPOSE 3000

# Run the binary
CMD ["./relay"]
