// Site identity colors are themed CSS custom properties (defined in
// global.sass as Washi/Sumi pairs); components receive the var() reference.
export const typeColors = {
  narou: 'var(--c-site-narou)',
  kakuyomu: 'var(--c-site-kakuyomu)',
  nocturne: 'var(--c-site-nocturne)',
}

export const navItems = [
  { label: 'favorite', path: '/', color: 'var(--c-nav-favorite)' },
  { label: 'narou', path: '/ranking/narou', color: typeColors.narou },
  { label: 'kakuyomu', path: '/ranking/kakuyomu', color: typeColors.kakuyomu },
  { label: 'nocturne', path: '/ranking/nocturne', color: typeColors.nocturne },
]
