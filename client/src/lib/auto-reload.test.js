import { describe, expect, it, vi } from 'vitest'
import { AUTO_RELOAD_INTERVAL_MS, startAutoReload } from './auto-reload.js'

function createHarness(initialVisibility = 'visible') {
  const listeners = new Map()
  const intervals = new Map()
  let nextIntervalId = 1

  const documentRef = {
    visibilityState: initialVisibility,
    addEventListener: vi.fn((type, listener) => listeners.set(type, listener)),
    removeEventListener: vi.fn((type, listener) => {
      if (listeners.get(type) === listener) listeners.delete(type)
    }),
  }
  const timers = {
    setInterval: vi.fn((callback, delay) => {
      const id = nextIntervalId++
      intervals.set(id, { callback, delay })
      return id
    }),
    clearInterval: vi.fn((id) => intervals.delete(id)),
  }

  return {
    documentRef,
    timers,
    intervals,
    setVisibility(state) {
      documentRef.visibilityState = state
      listeners.get('visibilitychange')?.()
    },
    tick() {
      for (const { callback } of intervals.values()) callback()
    },
    hasVisibilityListener() {
      return listeners.has('visibilitychange')
    },
  }
}

describe('startAutoReload', () => {
  it('reloads every 60 seconds while visible without duplicating the initial load', () => {
    const harness = createHarness()
    const reload = vi.fn()

    const stop = startAutoReload(reload, {
      documentRef: harness.documentRef,
      timers: harness.timers,
    })

    expect(reload).not.toHaveBeenCalled()
    expect(harness.intervals.size).toBe(1)
    expect([...harness.intervals.values()][0].delay).toBe(AUTO_RELOAD_INTERVAL_MS)

    harness.tick()
    expect(reload).toHaveBeenCalledOnce()
    stop()
  })

  it('pauses while hidden and reloads immediately when visible again', () => {
    const harness = createHarness()
    const reload = vi.fn()
    const stop = startAutoReload(reload, {
      documentRef: harness.documentRef,
      timers: harness.timers,
    })

    harness.setVisibility('hidden')
    expect(harness.intervals.size).toBe(0)
    harness.tick()
    expect(reload).not.toHaveBeenCalled()

    harness.setVisibility('visible')
    expect(reload).toHaveBeenCalledOnce()
    expect(harness.intervals.size).toBe(1)
    harness.tick()
    expect(reload).toHaveBeenCalledTimes(2)
    stop()
  })

  it('starts paused when hidden and removes browser hooks during cleanup', () => {
    const harness = createHarness('hidden')
    const reload = vi.fn()
    const stop = startAutoReload(reload, {
      documentRef: harness.documentRef,
      timers: harness.timers,
    })

    expect(harness.intervals.size).toBe(0)
    expect(harness.hasVisibilityListener()).toBe(true)

    stop()
    expect(harness.intervals.size).toBe(0)
    expect(harness.hasVisibilityListener()).toBe(false)
    harness.setVisibility('visible')
    expect(reload).not.toHaveBeenCalled()
  })
})
