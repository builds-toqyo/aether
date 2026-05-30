# Multi-stage Docker build for Aether
FROM node:22-alpine AS frontend-builder

WORKDIR /app

# Copy frontend files
COPY package*.json ./
COPY src/ ./src/
COPY next.config.mjs tsconfig.json ./

# Install frontend dependencies
RUN npm ci --only=production

# Build frontend
RUN npm run build

# Rust build stage
FROM rust:1.80-alpine AS backend-builder

# Install system dependencies for FFmpeg and GStreamer
RUN apk add --no-cache \
    gcc \
    musl-dev \
    pkgconfig \
    openssl-dev \
    ffmpeg-dev \
    gstreamer-dev \
    gstreamer-plugins-base-dev \
    gstreamer-editing-services-dev \
    python3 \
    make \
    g++

WORKDIR /app

# Copy Rust files
COPY Cargo.toml Cargo.lock ./
COPY src-tauri/ ./src-tauri/

# Install Tauri CLI
RUN cargo install tauri-cli --version ^2.0

# Copy frontend build
COPY --from=frontend-builder /app/out ./src-tauri/out

# Build Tauri app
RUN cargo tauri build --target x86_64-unknown-linux-musl

# Final runtime image
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache \
    libgcc \
    libstdc++ \
    ffmpeg \
    gstreamer \
    gstreamer-plugins-base \
    gstreamer-plugins-good \
    gstreamer-plugins-ugly \
    gstreamer-plugins-bad \
    gstreamer-editing-services \
    gtk+3.0 \
    libx11 \
    libxrandr \
    libxinerama \
    libxcursor \
    libxtst \
    libxi

# Create app user
RUN addgroup -g 1001 -S aether && \
    adduser -S aether -u 1001 -G aether

WORKDIR /app

# Copy built application
COPY --from=backend-builder /app/src-tauri/target/x86_64-unknown-linux-musl/release/bundle/appimage/*.AppImage /opt/aether.AppImage

# Make executable
RUN chmod +x /opt/aether.AppImage

# Switch to non-root user
USER aether

# Expose port for potential web interface
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD /opt/aether.AppImage --version || exit 1

# Set entrypoint
ENTRYPOINT ["/opt/aether.AppImage"]
