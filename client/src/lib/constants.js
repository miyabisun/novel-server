// Site identity colors are themed CSS custom properties (defined in
// global.sass as Washi/Sumi pairs); components receive the var() reference.
export const typeColors = {
  narou: 'var(--c-site-narou)',
  kakuyomu: 'var(--c-site-kakuyomu)',
  nocturne: 'var(--c-site-nocturne)',
}

// label is used for display (≥800px) and ranking type matching (site tabs).
// short is the compact mobile label (≤799px): root doubles as brand "novel"
// when the novel-server title is hidden; site tabs use 3-letter shorts.
export const navItems = [
  { label: 'favorite', short: 'novel', path: '/', color: 'var(--c-nav-favorite)' },
  { label: 'narou', short: 'nar', path: '/ranking/narou', color: typeColors.narou },
  { label: 'kakuyomu', short: 'kak', path: '/ranking/kakuyomu', color: typeColors.kakuyomu },
  { label: 'nocturne', short: 'noc', path: '/ranking/nocturne', color: typeColors.nocturne },
]

/** Swipe-hint text: prefer short so touch hints match compact header labels. */
export function navSwipeLabel(item) {
  return item.short ?? item.label
}
