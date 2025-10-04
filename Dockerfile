# Multi-stage build for housing web application

# Stage 1: Build the frontend
FROM node:20-alpine as frontend-builder

WORKDIR /app/webpage
COPY webpage/package*.json ./
RUN npm ci --only=production

COPY webpage/ ./
RUN npm run build-docker

# Stage 2: Build the backend
FROM rust:1.90-alpine as backend-builder

# Install required system dependencies for Alpine
RUN apk add --no-cache musl-dev pkgconfig openssl-dev

WORKDIR /app

# Copy workspace files and server dependencies
COPY server/Cargo.toml server/Cargo.lock ./
COPY server/macros/ ./macros/

# Create dummy main.rs files to build dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN mkdir -p macros/src && echo "fn main() {}" > macros/src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release
RUN rm -rf src macros/src

# Copy actual source code
COPY server/src ./src
COPY server/macros/src ./macros/src

# Build the application
RUN cargo build --release

# Stage 3: Runtime
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Create app user
RUN addgroup -g 1001 -S app && \
    adduser -S -D -H -u 1001 -s /bin/bash -G app app

WORKDIR /app

# Copy the built binary from backend builder
COPY --from=backend-builder /app/target/release/housing-webpage-login ./server

# Copy the built frontend from frontend builder
COPY --from=frontend-builder /app/webpage/dist ./webpage/dist

# Change ownership to app user
RUN chown -R app:app /app

# Switch to app user
USER app

# Expose the port
EXPOSE 8000

# Set environment variables
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

# Run the server
CMD ["./server"]
