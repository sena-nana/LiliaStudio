import { describe, expect, it } from 'vitest'
import { normalizeRichTextContent, textToRichTextDocument } from '@/components/richTextContent'

describe('rich text content helpers', () => {
  it('keeps boundary whitespace when wrapping plain text', () => {
    const parsed = JSON.parse(textToRichTextDocument('  第一行\n第二行\n'))

    expect(parsed.content[0].content[0].text).toBe('  第一行')
    expect(parsed.content[1].content[0].text).toBe('第二行')
    expect(parsed.content[2]).toEqual({ type: 'paragraph' })
  })

  it('treats whitespace-only text as content', () => {
    const parsed = JSON.parse(normalizeRichTextContent('  '))

    expect(parsed.content[0].content[0].text).toBe('  ')
  })

  it('accepts tiptap json surrounded by whitespace', () => {
    const document = '{"type":"doc","content":[{"type":"customNode"}]}'

    expect(normalizeRichTextContent(`  ${document}\n`)).toBe(document)
  })

  it('wraps doc-shaped json with invalid content as plain text', () => {
    const raw = '{"type":"doc","content":"说明"}'
    const parsed = JSON.parse(normalizeRichTextContent(raw))

    expect(parsed.content[0].content[0].text).toBe(raw)
  })
})
