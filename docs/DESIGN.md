---
version: alpha
name: Sumi / novel-server
description: >
  Self-contained design authority for novel-server. Adapts the home-server
  unified starter (rust-svelte-template / Sumi family) and records project
  accent, functional data colors, four-way theme contract, and domain chrome
  (nav tabs, ranking band, reader). Implemented in client/src/global.sass and
  client components; site identity colors also surface in
  client/src/lib/constants.js.
colors:
  # --- Project accent (blue) ---
  # Unsuffixed = Kinari (light) / Washi (e-paper) shared blue ink;
  # -dark = Sumi (dark).
  accent: "rgba(20, 100, 200, 0.95)"
  accent-subtle: "rgba(20, 100, 200, 0.1)"
  accent-dark: "rgba(128, 192, 255, 0.85)"
  accent-subtle-dark: "rgba(128, 192, 255, 0.15)"
  # --- Functional data colors ---
  fav: "rgba(180, 120, 0, 0.95)"
  fav-dark: "rgba(255, 200, 50, 0.8)"
  # Site identity. Washi/e-paper values are darkness-ramp inks; Sumi keeps
  # vivid 0.7-alpha hues.
  site-narou: "#1f6b3a"
  site-narou-dark: "rgba(100, 190, 120, 0.7)"
  site-kakuyomu: "#14508c"
  site-kakuyomu-dark: "rgba(100, 160, 220, 0.7)"
  site-nocturne: "#8f2b2b"
  site-nocturne-dark: "rgba(200, 110, 110, 0.7)"
  # CSS token remains --c-nav-favorite; tab label is "novel".
  nav-favorite: "#8a6a00"
  nav-favorite-dark: "rgba(220, 180, 50, 0.7)"
---

# novel-server — design authority

## Overview

novel-server uses the Sumi family (via `rust-svelte-template`) as the
home-server unified frontend base. Shared chrome recipes (header menu,
theme modal, icon dictionary, sticky sub-header band) are adapted here;
**domain semantics in this file win** on conflict (notably primary nav tabs
beside the hamburger).

Accent: **blue** (`rgba(20,100,200)` light/e-paper / `rgba(128,192,255)` Sumi).
It marks interactive chrome only: links, active pager buttons, focused
inputs, the shared focus ring.

### Themes (in-app toggle)

Four choices, key `novel-server:theme`, attribute `data-theme` on `<html>`:

| choice | palette | notes |
| --- | --- | --- |
| `system` | OS → Kinari (light) or Sumi (dark) | default; no attribute |
| `light` | Kinari | screen light (template neutrals + blue accent) |
| `dark` | Sumi | forces dark even when OS is light |
| `e-paper` | Washi | e-ink tuned; not the default light |

`:root` in `client/src/global.sass` IS Sumi. Washi is only applied when the
user selects **電子ペーパー** (`e-paper`).

Layout breakpoints stay at **799 / 800px**. Smartphone E2E and visual
checks use a **400px-wide** viewport.

## Colors

Functional data colors (domain state, not decoration):

- **Fav (gold):** favorited state only — add button, favorited row border,
  reader fav toggle. Washi uses dark amber ink.
- **Site identity (narou / kakuyomu / nocturne / novel gold):** active nav
  underline and type badge on favorite cards.

Danger uses the family danger role with no extra project meaning.

## Components

- **App header:** sticky 48px. Left: non-interactive `novel-server` title
  (hidden ≤799px) + **nav tabs** (domain override — tabs are primary
  navigation). Labels: `novel` (`/` favorites), `narou` / `kakuyomu` /
  `nocturne` (desktop); site tabs shorten to `nar` / `kak` / `noc` at
  ≤799px. Right: hamburger (36px). Menu order: optional auth email
  (read-only, Cloudflare Access), **テーマ設定** first action, then
  compact-viewport reader actions (目次 / お気に入りから削除).
- **Ranking control band:** sticky under header; same grey recipe as
  Reader (`.reader-bar`: `--c-surface`, hairline, `--subheader-h` 40px).
  Controls 36px tall; band must fully cover scrolling card text. Band
  controls use the shared 2px focus outline with a local
  `outline-offset: -1px` so the ring stays fully inside the 40px
  border-box band (shared `outline-offset: 2px` clips under the app
  header; offset 0 still leaves ~0.5px over the 1px bottom border).
- **Ranking card:** rank, page count, title link, action rail (detail /
  favorite). Favorited rows: 3px fav left border.
- **Favorite card:** progress, update time, type badge, title resume link,
  delete via ConfirmModal.
- **Reader:** sticky grey bar (title, prev/next, 目次, fav). On ≤799px,
  目次 and お気に入り削除 move into the app hamburger; お気に入り追加
  stays on the bar. Arrow keys and swipe turn pages; progress saves on
  load with no chrome feedback.
- **Icons:** monochrome `currentColor` SVG via `Icon.svelte`. Template
  paths for menu/x/sun/moon/monitor/book; `search` and `star-outline` /
  `star-filled` are project extensions.
- **Modals:** ConfirmModal, NovelDetailModal, ThemeModal (system / light /
  dark / e-paper).

## Do's and Don'ts

- Do keep gold monosemous: fav color = favorited state only.
- Do keep site colors monosemous: source identity only.
- Don't put `//` comments at the end of a Sass custom-property value line.
- Do route every destructive action through ConfirmModal.
- Do tune **Washi / e-paper** values on a real e-paper device — contrast
  over hue; e-paper is not a cosmetic light variant.
