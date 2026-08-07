/** @typedef {'system' | 'light' | 'dark' | 'e-paper'} ThemeChoice */

export const THEME_STORAGE_KEY = 'novel-server:theme'

/**
 * @param {string | null | undefined} value
 * @returns {value is Exclude<ThemeChoice, 'system'>}
 */
function isStoredChoice(value) {
  return value === 'light' || value === 'dark' || value === 'e-paper'
}

/** @returns {ThemeChoice} */
export function loadTheme() {
  try {
    const stored = window.localStorage.getItem(THEME_STORAGE_KEY)
    if (isStoredChoice(stored)) return stored
  } catch {
    // Storage can be unavailable (private mode); treat as system.
  }
  return 'system'
}

/** @param {ThemeChoice} choice */
export function applyTheme(choice) {
  if (choice === 'system') {
    delete document.documentElement.dataset.theme
  } else {
    document.documentElement.dataset.theme = choice
  }
}

/** @param {ThemeChoice} choice */
export function saveTheme(choice) {
  try {
    if (choice === 'system') {
      window.localStorage.removeItem(THEME_STORAGE_KEY)
    } else {
      window.localStorage.setItem(THEME_STORAGE_KEY, choice)
    }
  } catch {
    // Persisting is best-effort; still apply for this session.
  }
  applyTheme(choice)
}
