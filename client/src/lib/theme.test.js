import { afterEach, describe, expect, it } from 'vitest'
import { THEME_STORAGE_KEY, applyTheme, loadTheme, saveTheme } from './theme.js'

function installDomMocks() {
  const store = new Map()
  const dataset = {}
  globalThis.window = {
    localStorage: {
      getItem(key) {
        return store.has(key) ? store.get(key) : null
      },
      setItem(key, value) {
        store.set(key, String(value))
      },
      removeItem(key) {
        store.delete(key)
      },
      clear() {
        store.clear()
      },
    },
  }
  globalThis.document = {
    documentElement: {
      dataset,
    },
  }
  return { store, dataset }
}

describe('theme', () => {
  /** @type {{ store: Map<string, string>, dataset: Record<string, string> }} */
  let mocks

  afterEach(() => {
    mocks.store.clear()
    for (const key of Object.keys(mocks.dataset)) delete mocks.dataset[key]
  })

  it('defaults to system when nothing is stored', () => {
    mocks = installDomMocks()
    expect(loadTheme()).toBe('system')
  })

  it('saving light sets the attribute and persists the choice', () => {
    mocks = installDomMocks()
    saveTheme('light')

    expect(document.documentElement.dataset.theme).toBe('light')
    expect(window.localStorage.getItem(THEME_STORAGE_KEY)).toBe('light')
    expect(loadTheme()).toBe('light')
  })

  it('saving dark sets the attribute and persists the choice', () => {
    mocks = installDomMocks()
    saveTheme('dark')

    expect(document.documentElement.dataset.theme).toBe('dark')
    expect(window.localStorage.getItem(THEME_STORAGE_KEY)).toBe('dark')
    expect(loadTheme()).toBe('dark')
  })

  it('saving e-paper sets the attribute and persists the choice', () => {
    mocks = installDomMocks()
    saveTheme('e-paper')

    expect(document.documentElement.dataset.theme).toBe('e-paper')
    expect(window.localStorage.getItem(THEME_STORAGE_KEY)).toBe('e-paper')
    expect(loadTheme()).toBe('e-paper')
  })

  it('saving system removes both the attribute and the stored key', () => {
    mocks = installDomMocks()
    saveTheme('light')
    saveTheme('system')

    expect(document.documentElement.dataset.theme).toBeUndefined()
    expect(window.localStorage.getItem(THEME_STORAGE_KEY)).toBeNull()
    expect(loadTheme()).toBe('system')
  })

  it('ignores unknown stored values', () => {
    mocks = installDomMocks()
    window.localStorage.setItem(THEME_STORAGE_KEY, 'sepia')

    expect(loadTheme()).toBe('system')
  })

  it('applyTheme alone does not persist', () => {
    mocks = installDomMocks()
    applyTheme('light')

    expect(document.documentElement.dataset.theme).toBe('light')
    expect(window.localStorage.getItem(THEME_STORAGE_KEY)).toBeNull()
  })
})
