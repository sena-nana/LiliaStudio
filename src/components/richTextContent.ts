export const EMPTY_DOCUMENT = '{"type":"doc","content":[{"type":"paragraph"}]}'

export function parseRichTextContent(value: string) {
  return JSON.parse(normalizeRichTextContent(value))
}

export function normalizeRichTextContent(value: string) {
  if (!value) return EMPTY_DOCUMENT
  const trimmed = value.trim()
  try {
    const parsed = JSON.parse(trimmed) as unknown
    if (isDoc(parsed)) return JSON.stringify(parsed)
  } catch {
    return textToRichTextDocument(value)
  }
  return textToRichTextDocument(value)
}

function isDoc(value: unknown) {
  return (
    isRecord(value) &&
    value.type === 'doc' &&
    (value.content === undefined || Array.isArray(value.content))
  )
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

export function textToRichTextDocument(value: string) {
  const normalized = value.replace(/\r\n/g, '\n').replace(/\r/g, '\n')
  const content = normalized.split('\n').map((line) => {
    if (!line) return { type: 'paragraph' }
    return { type: 'paragraph', content: [{ type: 'text', text: line }] }
  })
  return JSON.stringify({ type: 'doc', content })
}
