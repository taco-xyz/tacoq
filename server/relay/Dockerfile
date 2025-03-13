# Base image for chef
FROM rust:1.82.0 AS chef
WORKDIR /server
RUN cargo install cargo-chef

# Planner stage
FROM chef AS planner
WORKDIR /app

# Copy workspace files first
COPY . .

# Create recipe
RUN cargo chef prepare --recipe-path recipe.json

# Builder stage
FROM chef AS builder
WORKDIR /app

# Install curl for health checks
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# First get the recipe
COPY --from=planner /app/recipe.json recipe.json

# Cook dependencies
RUN cargo chef cook --release --recipe-path recipe.json

# Copy the whole workspace structure
COPY . .

# Build the specific service
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin relay

# Strip the binary
RUN strip target/release/relay

# Final stage
FROM gcr.io/distroless/cc-debian12:nonroot AS release
WORKDIR /app

# Copy the binary from the correct location
COPY --from=builder /app/target/release/relay .

EXPOSE 3000

CMD ["./relay"]
