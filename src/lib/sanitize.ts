const ALLOWED_TAGS = new Set([
  'p', 'br', 'hr', 'div', 'span',
  'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
  'ruby', 'rt', 'rp', 'rb',
  'em', 'strong', 'b', 'i', 'u', 's', 'sub', 'sup',
])

// コンテンツごと完全削除するタグ（テキストが漏れると表示が壊れる）
const REMOVE_WITH_CONTENT = new Set([
  'script', 'style', 'title', 'noscript', 'template',
])

export function sanitizeHtml(html: string | null): string {
  if (!html) return ''
  return new HTMLRewriter()
    .onDocument({
      comments(comment) {
        comment.remove()
      },
    })
    .on('*', {
      element(el) {
        if (REMOVE_WITH_CONTENT.has(el.tagName)) {
          el.remove()
        } else if (ALLOWED_TAGS.has(el.tagName)) {
          for (const [name] of el.attributes) {
            el.removeAttribute(name)
          }
        } else {
          el.removeAndKeepContent()
        }
      },
    })
    .transform(html)
}
