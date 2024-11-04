# ====================
# Stage 1: Build Stage
# ====================
FROM rust:1.82.0-slim AS builder

# Install required dependencies for SQLite and OpenSSL
RUN apt-get update && \
    apt-get install -y libsqlite3-dev libssl-dev pkg-config perl make && \
    rm -rf /var/lib/apt/lists/*

# Add the wasm32 target for Dioxus
RUN rustup target add wasm32-unknown-unknown

# Install Dioxus CLI
RUN cargo install dioxus-cli && dx --version

# Set the working directory
WORKDIR /app

# Copy Cargo files first to leverage Docker's layer caching for dependencies
COPY Cargo.toml Cargo.lock ./

# Copy the entire source code
COPY . .

# Build the application in release mode
RUN dx build --release

# ======================
# Stage 2: Runtime Stage
# ======================
FROM debian:bookworm-slim

# Install dependencies required for SQLite and OpenSSL 3
RUN apt-get update && \
    apt-get install -y libsqlite3-dev openssl libssl-dev pkg-config && \
    rm -rf /var/lib/apt/lists/*

# Set up the working directory for the app
WORKDIR /app

# Copy the compiled application and necessary files from the builder stage
COPY --from=builder /app/dist /app/dist
COPY --from=builder /app/.env /app/

# Expose the port the app will run on (adjust if necessary)
EXPOSE 8080

# Command to run the application
CMD ["./dist/blogposts"]
