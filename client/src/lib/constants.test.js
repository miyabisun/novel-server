import { describe, expect, it } from 'vitest'
import { navItems } from './constants.js'

describe('navItems', () => {
  it('renames the root tab to novel without changing its path', () => {
    expect(navItems[0]).toMatchObject({ label: 'novel', path: '/' })
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

  it('does not shorten the novel tab (site name stays full)', () => {
    expect(navItems[0].short).toBeUndefined()
  })
})
