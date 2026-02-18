# novel-server

> [日本語ドキュメントはこちら](README.ja.md)

A novel ranking viewer & reader supporting Narou, Kakuyomu, and Nocturne.
Includes favorites management and automatic reading progress tracking.

## Quick Start (Docker)

```bash
docker run -p 3000:3000 -v novel-data:/data ghcr.io/miyabisun/novel-server:latest
```

Open `http://localhost:3000` in your browser.

## Quick Start (Node.js)

```bash
npm run setup && npm run build:client
npm start
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

- **Rankings** — Browse rankings from Narou / Kakuyomu / Nocturne with period selection and synopsis preview
- **Reader** — Keyboard-navigable (arrow keys) page turning
- **Favorites** — Add from rankings with ★, manage and remove from list
- **Reading Progress** — Automatically saved when a page loads in the reader

## Documentation

- [Development Guide](docs/development.md) — Local setup, build, and project structure
- [API Reference](docs/api.md) — REST API specification
- [Architecture](docs/architecture.md) — Backend / frontend architecture overview
