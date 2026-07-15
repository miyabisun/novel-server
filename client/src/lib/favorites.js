import config from '$lib/config.js'
import fetcher from '$lib/fetcher.js'

function favoriteUrl(type, id) {
  return `${config.path.api}/favorites/${type}/${id}`
}

export function addFavorite(type, id, favorite) {
  return fetcher(favoriteUrl(type, id), {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ title: favorite.title, page: favorite.page }),
  })
}

export function removeFavorite(type, id) {
  return fetcher(favoriteUrl(type, id), { method: 'DELETE' })
}
