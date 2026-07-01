import stringWidth from 'string-width'

export type LogLine = { id: number; text: string; color?: string }
export type DisplayLine = { key: string; text: string; color?: string }

export function normalizeLogText(text: string) {
  return text.replace(/\s+/g, ' ').trim()
}

export function logContentWidth(columns: number) {
  return Math.max(20, columns - 12)
}

export function wrapLogText(text: string, width: number): string[] {
  if (width < 1) return [text]

  const lines: string[] = []
  for (const paragraph of text.replace(/\r\n/g, '\n').split('\n')) {
    if (!paragraph) {
      lines.push('')
      continue
    }

    let current = ''
    let currentWidth = 0

    const pushLine = () => {
      if (!current) return
      lines.push(current.trimEnd())
      current = ''
      currentWidth = 0
    }

    const pushChars = (value: string) => {
      for (const char of value) {
        const charWidth = stringWidth(char)
        if (currentWidth + charWidth > width && currentWidth > 0) {
          pushLine()
        }
        current += char
        currentWidth += charWidth
      }
    }

    for (const word of paragraph.split(/(\s+)/)) {
      if (!word) continue
      const wordWidth = stringWidth(word)

      if (wordWidth > width) {
        pushLine()
        pushChars(word)
        continue
      }

      if (currentWidth + wordWidth > width && currentWidth > 0) {
        pushLine()
      }

      current += word
      currentWidth += wordWidth
    }

    pushLine()
  }

  return lines.length > 0 ? lines : ['']
}

export function expandLogLines(logs: LogLine[], width: number): DisplayLine[] {
  const expanded: DisplayLine[] = []
  for (const log of logs) {
    const parts = wrapLogText(log.text, width)
    parts.forEach((text, index) => {
      expanded.push({ key: `${log.id}:${index}`, text, color: log.color })
    })
  }
  return expanded
}
