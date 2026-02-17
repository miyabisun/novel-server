# novel-server

A novel ranking viewer & reader supporting Narou, Kakuyomu, and Nocturne.
Includes favorites management and automatic reading progress tracking.

[日本語](README.ja.md)

## Tech Stack

- **Backend**: Hono + TypeScript + Prisma (SQLite)
- **Frontend**: Svelte 5 + Vite + Sass
- **Auth**: JWT (HS256) + HttpOnly Cookie

## Setup

Run the following command to install dependencies, generate `.env`, and initialize the database:

```bash
./scripts/install.sh
```

You will be prompted to set `AUTH_PASSWORD` only if `.env` does not exist (leave blank to auto-generate).
Safe to re-run on an already configured environment.

### Manual Setup

To run each step individually:

```bash
npm install                # Server dependencies
cd client && npm install   # Client dependencies
npm run init               # Generate .env interactively
npm run db:push            # Sync Prisma schema to DB
```

To create `.env` manually, refer to `.env.example`.
The server will fail to start if `AUTH_USERNAME`, `AUTH_PASSWORD`, or `JWT_SECRET` is not set.

## Running

```bash
# Development (with frontend hot rebuild)
npm run dev

# Production
npm run build
npm start
```

Open `http://localhost:3000` to see the login screen.

## Features

- **Rankings** — Browse rankings from Narou / Kakuyomu / Nocturne with period selection (daily / weekly / monthly / quarter / yearly)
- **Reader** — Keyboard-navigable (arrow keys) page turning
- **Favorites** — Add from rankings with ★, manage and remove from list
- **Reading Progress** — Automatically saved when a page loads in the reader
- **Auth** — All APIs require login (single user)

## Commands

| Command | Description |
|---------|-------------|
| `./scripts/install.sh` | Run full initial setup |
| `npm run dev` | Start dev server (vite watch + node) |
| `npm run build` | Build frontend |
| `npm start` | Start production server |
| `npm run db:push` | Sync Prisma schema to DB |
| `npm run init` | Generate `.env` interactively |

## Reverse Proxy

Set `BASE_PATH` to serve under a subpath:

```env
BASE_PATH=/novels
```

## Documentation

- [API Reference](docs/api.md)
- [Authentication](docs/authentication.md)
- [Architecture](docs/architecture.md)
