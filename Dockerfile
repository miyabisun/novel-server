# Stage 1: Frontend build
FROM oven/bun:1-slim AS frontend
WORKDIR /app/client
COPY client/package.json client/bun.lock ./
RUN bun install --frozen-lockfile
COPY client/ .
RUN bun run build

# Stage 2: Rust build
FROM rust:1-slim AS backend
RUN apt-get update && apt-get install -y pkg-config libssl-dev curl && rm -rf /var/lib/apt/lists/*
WORKDIR /app
# Build dependencies first (cached unless Cargo.toml/Cargo.lock change)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src
# Build the actual source (only this layer re-runs on code changes)
# touch: Docker COPY preserves host timestamps, which may be older than
# the cached dependency build above, causing cargo to skip the rebuild.
COPY src/ src/
RUN touch src/main.rs && cargo build --release

# Stage 3: Production runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=backend /app/target/release/novel-server /usr/local/bin/
COPY --from=frontend /app/client/build /app/client/build
WORKDIR /app
ENV PORT=3000
EXPOSE 3000
CMD ["novel-server"]
