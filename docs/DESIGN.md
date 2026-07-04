---
version: alpha
name: Sumi / novel-server
description: >
  novel-server project overrides for the Sumi design system. The canonical
  template lives at ~/.claude/designs/sumi/DESIGN.md; this file records
  ONLY what is specific to novel-server (accent + functional data colors +
  domain components). CSS custom properties in client/src/global.sass are
  the implementation of these tokens; site identity colors also surface in
  client/src/lib/constants.js.
colors:
  # --- Project accent (blue) ---
  # Unsuffixed = Washi theme (light), -dark = Sumi theme (dark).
  # Values are rgba because the accent family (hover / subtle / focus ring)
  # derives from one rgb at different alphas.
  accent: "rgba(20, 100, 200, 0.95)"
  accent-subtle: "rgba(20, 100, 200, 0.1)"
  accent-dark: "rgba(128, 192, 255, 0.85)"
  accent-subtle-dark: "rgba(128, 192, 255, 0.15)"
  # --- Functional data colors ---
  # fav = the "favorited" state.
  fav: "rgba(180, 120, 0, 0.95)"
  fav-dark: "rgba(255, 200, 50, 0.8)"
  # Site identity colors. Washi values are darkness-ramp inks (provisional
  # seeds — tune on an e-paper device); Sumi keeps the vivid 0.7-alpha hues.
  site-narou: "#1f6b3a"
  site-narou-dark: "rgba(100, 190, 120, 0.7)"
  site-kakuyomu: "#14508c"
  site-kakuyomu-dark: "rgba(100, 160, 220, 0.7)"
  site-nocturne: "#8f2b2b"
  site-nocturne-dark: "rgba(200, 110, 110, 0.7)"
  nav-favorite: "#8a6a00"
  nav-favorite-dark: "rgba(220, 180, 50, 0.7)"
---

# novel-server — Sumi Project Overrides

## Overview

**This project follows the Sumi design system.** The canonical template is
`~/.claude/designs/sumi/DESIGN.md` — all shared rules (neutral chrome,
one-accent rule, scales, flat elevation, iconography, component recipes)
live there and are NOT restated here. This document records only what is
unique to novel-server. On chrome questions the template wins; on the
domain semantics below this file wins.

Accent: **blue** (`rgba(20,100,200)` Washi / `rgba(128,192,255)` Sumi).
Blue distinguishes novel-server from its amber sibling (5ch-viewer) at a
glance. It marks interactive chrome only: links, active pager buttons,
focused inputs, the shared focus ring.

Themes follow the template's Sumi-first rule: `:root` in
`client/src/global.sass` IS the Sumi (dark) theme, and Washi (light,
e-paper) is applied via `@media (prefers-color-scheme: light)` — the OS
decides; there is no in-app toggle and no `data-theme` attribute.

## Colors

Everything below is a **functional data color** in the Sumi sense: it
encodes domain state, never decoration, and is exempt from the one-accent
rule.

- **Fav (rgba(180,120,0) / rgba(255,200,50)):** Gold means exactly one
  thing — "this novel is favorited". It appears as: the ☆→★ add button and
  its hover, the 3px left border on already-favorited ranking rows, the
  favorite toggle in the reader bar. The Washi value is a dark amber ink
  (bright gold is invisible on e-paper).
- **Site identity (narou green / kakuyomu blue / nocturne red / favorite
  gold):** Each content source owns one hue, used in exactly two places:
  the 2px underline of the active nav tab, and the outlined type badge on
  favorite cards. Washi values are darkness-ramp inks per the template
  (ink first, hue as a secondary cue) — provisional seeds pending e-paper
  tuning. Sumi keeps the vivid 0.7-alpha hues; the caption-size badge sits
  near the AA contrast boundary there, so darken the badge, not the hue,
  if it proves hard to read.

Danger (delete actions) uses the template's danger role with no extra
project meaning.

## Components

Domain components on top of the Sumi recipes:

- **Nav tabs (Header):** Sumi tab recipe, except the active underline is
  the destination's site identity color, not the accent — the nav doubles
  as the site legend. Inactive tabs stay muted.
- **Ranking card:** rank number, page count (短編 badge for one-shots),
  decoded title link, and a right action rail (detail, favorite add /
  remove). Favorited rows carry the 3px fav left border. On mobile, swipe
  right adds / swipe left removes a favorite, with the quiet swipe-hint
  panel per the template's gesture rule.
- **Favorite card:** progress (`read / page`) and last-update timestamp in
  muted caption, site type badge (outlined, site color), title link that
  resumes at `read + 1`, delete action behind ConfirmModal.
- **Reader:** sticky top bar holding title, prev/next pager (text-label
  buttons — 前 / 次 / 目次 — kept deliberately), and the fav toggle (fav
  color). Arrow keys and swipe both turn pages; progress saves
  automatically on page load with no UI feedback. Body text is the reading
  surface — chrome never competes with it.
- **Icons:** monochrome `currentColor` SVG via the shared `Icon.svelte`
  component, per the template.
- **Modals:** ConfirmModal (destructive confirmations) and
  NovelDetailModal (synopsis preview) follow the Sumi modal recipe.

## Do's and Don'ts

- Do keep gold monosemous: fav color = favorited state, nowhere else.
- Do keep site colors monosemous: source identity (nav underline, type
  badge), never chrome, never status.
- Don't write `//` comments at the end of a Sass custom-property value
  line — Sass leaves them inside the value and the property silently
  breaks. Comment on the line above.
- Do route every destructive action through ConfirmModal.
- Do tune Washi values on an actual e-paper device — contrast over hue;
  the light theme is not a cosmetic variant.
