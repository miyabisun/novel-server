import { describe, expect, it } from 'vitest'
import { navItems, navSwipeLabel } from './constants.js'

describe('navItems', () => {
  it('shows favorite on desktop and novel as compact short for the root tab', () => {
    expect(navItems[0]).toMatchObject({
      label: 'favorite',
      short: 'novel',
      path: '/',
    })
  })

  it('keeps ranking type labels and adds 3-letter mobile shorts', () => {
    const byLabel = Object.fromEntries(navItems.map((item) => [item.label, item]))
    expect(byLabel.narou.short).toBe('nar')
    expect(byLabel.kakuyomu.short).toBe('kak')
    expect(byLabel.nocturne.short).toBe('noc')
    expect(byLabel.narou.path).toBe('/ranking/narou')
    expect(byLabel.kakuyomu.path).toBe('/ranking/kakuyomu')
    expect(byLabel.nocturne.path).toBe('/ranking/nocturne')
  })
})

describe('navSwipeLabel', () => {
  it('prefers short over label so swipe hints match compact header text', () => {
    expect(navSwipeLabel(navItems[0])).toBe('novel')
    expect(navSwipeLabel(navItems[0])).not.toBe('favorite')
    expect(navSwipeLabel(navItems[1])).toBe('nar')
  })

  it('falls back to label when short is absent', () => {
    expect(navSwipeLabel({ label: 'favorite' })).toBe('favorite')
  })
})
