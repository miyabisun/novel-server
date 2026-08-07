/**
 * Reader page surfaces a few actions in the global hamburger on compact viewports.
 * Reader owns confirm UI and state; Header only invokes registered callbacks.
 */
export const readerChrome = $state({
  active: false,
  showToc: false,
  showUnfav: false,
  /** @type {null | (() => void)} */
  goToc: null,
  /** @type {null | (() => void)} */
  requestUnfav: null,
})

/** @param {Partial<typeof readerChrome>} partial */
export function updateReaderChrome(partial) {
  Object.assign(readerChrome, partial)
}

export function clearReaderChrome() {
  readerChrome.active = false
  readerChrome.showToc = false
  readerChrome.showUnfav = false
  readerChrome.goToc = null
  readerChrome.requestUnfav = null
}
