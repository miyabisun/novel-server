# Stage 1: Frontend build
FROM node:22-slim AS frontend
WORKDIR /app/client
COPY client/package.json client/package-lock.json ./
RUN npm ci
COPY client/ .
RUN npx vite build

# Stage 2: Nim build
FROM nimlang/nim:2.0.8 AS backend
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY novel_server.nimble nim.cfg ./
COPY src/ src/
RUN nimble install -y --depsOnly && nim c -d:release src/main.nim

# Stage 3: Production runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 libpcre3 libsqlite3-0 && rm -rf /var/lib/apt/lists/*
COPY --from=backend /app/src/main /usr/local/bin/novel-server
COPY --from=frontend /app/client/build /app/client/build
WORKDIR /app
ENV PORT=3000
EXPOSE 3000
CMD ["novel-server"]
