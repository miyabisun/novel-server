// Site identity colors are themed CSS custom properties (defined in
// global.sass as Washi/Sumi pairs); components receive the var() reference.
export const typeColors = {
  narou: 'var(--c-site-narou)',
  kakuyomu: 'var(--c-site-kakuyomu)',
  nocturne: 'var(--c-site-nocturne)',
}

// label is used for display (desktop) and ranking type matching (site tabs).
// short is the compact mobile label (3 letters) for site tabs only.
export const navItems = [
  { label: 'novel', path: '/', color: 'var(--c-nav-favorite)' },
  { label: 'narou', short: 'nar', path: '/ranking/narou', color: typeColors.narou },
  { label: 'kakuyomu', short: 'kak', path: '/ranking/kakuyomu', color: typeColors.kakuyomu },
  { label: 'nocturne', short: 'noc', path: '/ranking/nocturne', color: typeColors.nocturne },
]
