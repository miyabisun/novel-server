# novel-server

> [日本語ドキュメントはこちら](README.ja.md)

A novel ranking viewer & reader supporting Narou, Kakuyomu, and Nocturne.
Includes favorites management and automatic reading progress tracking.

## Quick Start (Docker)

```bash
docker run -p 3000:3000 -v novel-data:/data ghcr.io/miyabisun/novel-server:latest
```

Open `http://localhost:3000` in your browser.

## Quick Start (Rust)

```bash
cd client && npm install && npm run build && cd ..
cargo run --release
```

Open `http://localhost:3000` in your browser.

> To deploy under a reverse proxy subpath, set the `BASE_PATH` environment variable.

## Configuration

| Variable | Default | Description |
|---|---|---|
| `DATABASE_PATH` | `/data/novel.db` | SQLite database file path |
| `PORT` | `3000` | Server port |
| `BASE_PATH` | (empty) | Path prefix for reverse proxy deployment (e.g., `/novels`). Runtime only — no rebuild needed. |

The database is automatically created on first startup.

## Features

- **Rankings** — Browse rankings with period selection, synopsis preview, and swipe-to-add/remove favorites on mobile
- **Reader** — Keyboard-navigable (arrow keys) page turning
- **Favorites** — Add from rankings with ★, auto-sync metadata (page count, update time)
- **Reading Progress** — Automatically saved when a page loads in the reader
- **Multi-user** — When deployed behind an OAuth2 proxy that sets `X-Forwarded-Email`, favorites and reading progress are automatically scoped per user. Without the header, operates as a single guest user (backward compatible).

## Documentation

- [Development Guide](docs/development.md) — Local setup, build, and project structure
- API Reference — Available at `/swagger-ui/` when the server is running
