const friendlyMessages = {
  502: 'サイトに接続できませんでした。サイトがメンテナンス中の可能性があります。',
  503: 'サイトが一時的に利用できません。しばらくしてからお試しください。',
  504: 'サイトからの応答がタイムアウトしました。しばらくしてからお試しください。',
}

export default (url, options = {}) =>
  fetch(url, options).then(async (r) => {
    if (!r.ok) {
      const friendly = friendlyMessages[r.status]
      if (friendly) throw new Error(friendly)
      const body = await r.json().catch(() => null)
      throw new Error(body?.error || `${r.status} ${r.statusText}`)
    }
    return r.json()
  })
