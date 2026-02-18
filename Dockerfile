# Stage 1: Build
FROM node:20-alpine AS builder
WORKDIR /app

# Server dependencies
COPY package.json package-lock.json ./
RUN npm ci

# Prisma generate
COPY prisma/ ./prisma/
RUN ./node_modules/.bin/prisma generate

# Client build
COPY client/package.json client/package-lock.json ./client/
RUN cd client && npm ci
COPY client/ ./client/
RUN cd client && npm run build

# Assemble /dist
RUN npm prune --omit=dev && \
    mkdir /dist && \
    cp -r node_modules prisma package.json /dist/ && \
    mkdir -p /dist/client && \
    cp -r client/build /dist/client/
COPY src/ /dist/src/

# Stage 2: Production runtime
FROM node:20-alpine
WORKDIR /app
COPY --from=builder /dist ./
ENV NODE_ENV=production
ENV PORT=3000
EXPOSE 3000
CMD ["node", "--import", "tsx", "src/index.ts"]
