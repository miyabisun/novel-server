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

Without `PUBLIC_BASE_URL`, the origin uses `X-Forwarded-Proto` (default `http`),
`X-Forwarded-Host` or `Host` (default `localhost:<PORT>`), then appends `BASE_PATH`.
The database file is created on first startup; its parent directory is not created by the app.
The listener binds to `0.0.0.0`; Dockerfile explicitly sets `PORT=3000`.
For deployment values and volume mounts, see the
[home-server README](https://github.com/miyabisun/home-server/blob/main/README.md) and
[sis/compose.yaml](https://github.com/miyabisun/home-server/blob/main/sis/compose.yaml).

Sources: [configuration](src/config.rs), [startup](src/main.rs),
[SPA cache](src/spa.rs), [feed URL resolution](src/routes/feed.rs).

## Features

- **Rankings** — Browse rankings with period selection, synopsis preview, and swipe-to-add/remove favorites on mobile
- **Reader** — Keyboard-navigable (arrow keys) page turning
- **Favorites** — Add from rankings with ★, auto-sync metadata (page count, update time)
- **Reading Progress** — Automatically saved when a page loads in the reader
- **Multi-user** — When deployed behind Cloudflare Access that sets `Cf-Access-Authenticated-User-Email`, favorites and reading progress are automatically scoped per user. Without the header, operates as a single guest user.

## Documentation

- [Development Guide](docs/development.md) — Local setup, build, and project structure
- API Reference — Available at `/swagger-ui/` when the server is running
