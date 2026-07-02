export type WikiPage = {
  slug: string
  title: string
  content: string
  updatedBy: string
  createdAt: string
  updatedAt: string
}

const MAX_WIKI_PAGES = 200
export const MAX_WIKI_CONTENT_LENGTH = 20_000

const pages = new Map<string, WikiPage>()

export function wikiSlug(titleOrSlug: string): string {
  return titleOrSlug
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
}

export function writeWikiPage(input: { title: string; content: string; updatedBy: string }): WikiPage {
  const title = input.title.trim()
  const content = input.content.trim()
  const slug = wikiSlug(title)
  if (title.length < 2 || title.length > 120) throw new Error('Wiki page title must be 2-120 characters.')
  if (!slug) throw new Error('Wiki page title must contain at least one letter or number.')
  if (content.length < 1 || content.length > MAX_WIKI_CONTENT_LENGTH) {
    throw new Error(`Wiki page content must be 1-${MAX_WIKI_CONTENT_LENGTH} characters.`)
  }

  const now = new Date().toISOString()
  const existing = pages.get(slug)
  if (!existing && pages.size >= MAX_WIKI_PAGES) {
    throw new Error(`The wiki is full (${MAX_WIKI_PAGES} pages). Delete or update an existing page instead.`)
  }
  const page: WikiPage = {
    slug,
    title,
    content,
    updatedBy: input.updatedBy,
    createdAt: existing?.createdAt ?? now,
    updatedAt: now,
  }
  pages.set(slug, page)
  return { ...page }
}

export function readWikiPage(titleOrSlug: string): WikiPage | undefined {
  const page = pages.get(wikiSlug(titleOrSlug))
  return page ? { ...page } : undefined
}

export function deleteWikiPage(titleOrSlug: string): WikiPage | undefined {
  const slug = wikiSlug(titleOrSlug)
  const page = pages.get(slug)
  if (!page) return undefined
  pages.delete(slug)
  return { ...page }
}

export function listWikiPages(): WikiPage[] {
  return [...pages.values()]
    .map(page => ({ ...page }))
    .sort((a, b) => a.title.localeCompare(b.title))
}

// Used by team file load to restore a saved office wholesale.
export function replaceWikiPages(next: WikiPage[]): void {
  pages.clear()
  for (const page of next) pages.set(page.slug, { ...page })
}
