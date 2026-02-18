import fs from 'fs'
import path from 'path'

const indexPath = path.join(process.cwd(), 'client/build/index.html')
const isProd = process.env.NODE_ENV === 'production'

let indexHtml: string | null = null
let indexMtime = 0

export function getIndexHtml(basePath: string): string | null {
  if (isProd && indexHtml) return indexHtml

  try {
    const stat = fs.statSync(indexPath)
    const mtime = stat.mtimeMs
    if (!indexHtml || mtime !== indexMtime) {
      indexHtml = fs.readFileSync(indexPath, 'utf-8')
        .replace('<head>', `<head>\n\t\t<base href="${basePath}/">`)
        .replace('window.__BASE_PATH__ = ""', `window.__BASE_PATH__ = ${JSON.stringify(basePath)}`)
      indexMtime = mtime
    }
  } catch {
    return null
  }
  return indexHtml
}
