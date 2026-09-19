# novel-server

> [日本語ドキュメントはこちら](README.ja.md)

A novel ranking viewer & reader supporting Narou, Kakuyomu, and Nocturne.
Includes favorites management and automatic reading progress tracking.

## Quick Start (Docker)

Requires Docker. The named volume keeps favorites and reading progress across container restarts.

```bash
docker run -p 127.0.0.1:3000:3000 -v novel-data:/data ghcr.io/miyabisun/novel-server:latest
```

Open `http://localhost:3000` in your browser.

## Quick Start (Rust)

Requires Git, Rust, Node.js/npm, and the platform build tools for OpenSSL and SQLite.
The Docker build uses Rust 1.96; the frontend requires Node.js 20.19+ or 22.12+.

```bash
git clone https://github.com/miyabisun/novel-server.git
cd novel-server
cd client && npm install && npm run build && cd ..
DATABASE_PATH=./novel.db cargo run --release --locked
```

Open `http://localhost:3000` in your browser.

The database is created as `novel.db` in the checkout. Keep this file and its WAL/SHM files.
For another location, create a writable parent directory before setting `DATABASE_PATH`.

After opening the app, choose a site and ranking period, then open a novel.
Use ★ to add a favorite. Reading progress is saved when a reader page loads.

## Deployment and storage

The Docker example exposes only localhost. The native server listens on all interfaces.
There is no built-in login: without an identity header, users share the guest account.
For remote access, configure access control at your proxy or network boundary.
When using the Cloudflare identity header, prevent direct access around that proxy.

Mount the entire data directory, not just the SQLite file, to preserve WAL/SHM files.
See [SQLite operations](docs/development.md#sqlite-運用) for backup and old-database restoration.

For a reverse proxy subpath, set `BASE_PATH`, for example `/novels`.
See the [proxy examples](docs/development.md#リバースプロキシ).

## Environment Variables

The server loads `.env` at startup; existing process environment values take precedence.
All variables are optional. These are application defaults, including for a native release build.

| Variable | Required | Unset default | Purpose and invalid/empty values |
| --- | --- | --- | --- |
| `PORT` | No | `3000` | Listen port. Values that cannot parse as `u16` (including empty, negative, or above 65535) fall back to 3000. `0` lets the OS choose a port; bind failure stops startup. |
| `DATABASE_PATH` | No | `/data/novel.db` | SQLite file. The parent directory must exist and be writable; failure to open the DB stops startup. An empty value is passed directly to SQLite. |
| `BASE_PATH` | No | Empty | Runtime reverse proxy prefix, e.g. `/novels`. Trailing `/` characters are removed; empty or `/` means the root. Other values must match `^/[\w\-/]*$`; invalid values stop startup. No rebuild is needed. |
| `PUBLIC_BASE_URL` | No | Request origin plus `BASE_PATH` | Public base URL for feed links, e.g. `https://novel.example.com/novels`. Include the subpath yourself when set. Surrounding whitespace and trailing `/` are removed; empty falls back to the request. Other values are used without URL validation, so malformed values produce malformed links. |
| `NODE_ENV` | No | Development cache behavior | Exactly `production` keeps the cached SPA HTML even after its modification time changes. Every other value, including empty, uses modification-time refresh. This variable does not control authentication. |
| `RUST_LOG` | No | `info` | Log level or target filter, e.g. `debug` or `novel_server=debug`. Empty selects `error`. Invalid filter syntax prints a warning to stderr and disables logs. `LOG_LEVEL` is not read. |

Without `PUBLIC_BASE_URL`, the origin uses `X-Forwarded-Proto` (default `http`).
The host comes from `X-Forwarded-Host`, then `Host`, then `localhost:<PORT>`.
`BASE_PATH` is appended to that origin.
The database file is created on first startup. Its parent directory must already exist.
The listener binds to `0.0.0.0`; Dockerfile explicitly sets `PORT=3000`.
The Docker example mounts a named volume at `/data`, where the default database is stored.

Sources: [configuration](src/config.rs), [startup](src/main.rs),
[SPA cache](src/spa.rs), [feed URL resolution](src/routes/feed.rs).

## Features

- **Rankings** — Select a period and preview synopses. Swipe to manage favorites on mobile.
- **Reader** — Keyboard-navigable (arrow keys) page turning
- **Favorites** — Add from rankings with ★, auto-sync metadata (page count, update time)
- **Reading Progress** — Automatically saved when a page loads in the reader
- **Multi-user** — Favorites and reading progress are scoped per user.
  Cloudflare Access supplies the identity via `Cf-Access-Authenticated-User-Email`.
  Without the header, everyone uses the guest account.

## Documentation

- [Development Guide](docs/development.md) — Local setup, build, and project structure
- API Reference — Available at `/swagger-ui/` when the server is running
