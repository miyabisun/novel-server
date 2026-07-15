import { beforeEach, describe, expect, it, vi } from 'vitest'

const { fetcher } = vi.hoisted(() => ({ fetcher: vi.fn() }))

vi.mock('$lib/config.js', () => ({
  default: { path: { api: '/base/api' } },
}))
vi.mock('$lib/fetcher.js', () => ({ default: fetcher }))

import { addFavorite, removeFavorite } from './favorites.js'

beforeEach(() => {
  fetcher.mockReset()
})

describe('favorite API', () => {
  it('adds a favorite with the shared request contract', async () => {
    const response = { id: 'n1234ab' }
    fetcher.mockResolvedValue(response)

    await expect(addFavorite('narou', 'n1234ab', { title: 'Novel', page: 42 })).resolves.toBe(
      response,
    )
    expect(fetcher).toHaveBeenCalledWith('/base/api/favorites/narou/n1234ab', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ title: 'Novel', page: 42 }),
    })
  })

  it('removes a favorite with the shared request contract', async () => {
    fetcher.mockResolvedValue({ ok: true })

    await removeFavorite('kakuyomu', 'work-id')

    expect(fetcher).toHaveBeenCalledWith('/base/api/favorites/kakuyomu/work-id', {
      method: 'DELETE',
    })
  })

  it('preserves request failures for callers to handle', async () => {
    const error = new Error('failed')
    fetcher.mockRejectedValue(error)

    await expect(removeFavorite('narou', 'n1')).rejects.toBe(error)
  })
})
