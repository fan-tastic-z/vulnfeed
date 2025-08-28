# Build stage for frontend
FROM node:18-alpine AS frontend-build

# Install specific version of pnpm that matches the lockfile
RUN npm install -g pnpm@8

WORKDIR /app

# Copy package files first for better caching
COPY assets/package.json assets/pnpm-lock.yaml ./

# Install dependencies - this will be cached if package files don't change
RUN pnpm install --frozen-lockfile

# Copy source code after dependencies are installed
COPY assets/ .

# Build frontend
RUN pnpm build

# Build stage for backend with better caching
FROM rust:1.89-alpine AS backend-build

# Change to domestic mirror for faster apk installs
RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.tuna.tsinghua.edu.cn/g' /etc/apk/repositories

# Install system dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    postgresql-dev \
    git

WORKDIR /app

# Configure domestic mirror for faster crate downloads
RUN mkdir -p .cargo && \
    echo '[source.crates-io]' > .cargo/config.toml && \
    echo 'replace-with = "tuna"' >> .cargo/config.toml && \
    echo '[source.tuna]' >> .cargo/config.toml && \
    echo 'registry = "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git"' >> .cargo/config.toml && \
    echo '[net]' >> .cargo/config.toml && \
    echo 'git-fetch-with-cli = true' >> .cargo/config.toml

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock build.rs ./

# Copy source code
COPY src/ src/

# Copy frontend build output
COPY --from=frontend-build /app/public/ public/

# Build the application
RUN cargo build --release

# Production stage
FROM alpine:latest

# Change to domestic mirror for faster apk installs
RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.tuna.tsinghua.edu.cn/g' /etc/apk/repositories

# Install runtime dependencies
RUN apk add --no-cache ca-certificates wget

WORKDIR /app

# Create non-root user
RUN adduser -D -s /bin/sh app

# Copy binary
COPY --from=backend-build /app/target/release/vulnfeed ./vulnfeed

# Copy default config
COPY dev/config.toml ./config.toml

# Set permissions
RUN chown -R app:app ./
USER app

# Expose port
EXPOSE 9000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD wget --quiet --tries=1 --spider http://localhost:9000/ || exit 1

# Run the application
CMD ["./vulnfeed", "server", "--config-file", "config.toml"]