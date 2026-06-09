import { mount } from '@vue/test-utils'
import { afterEach, describe, expect, it } from 'vitest'
import { nextTick } from 'vue'
import RichTextEditor from '@/components/RichTextEditor.vue'

const initialDocument = JSON.stringify({
  type: 'doc',
  content: [
    {
      type: 'paragraph',
      content: [{ type: 'text', text: '初始内容' }],
    },
  ],
})

describe('RichTextEditor', () => {
  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('renders initial tiptap json content', async () => {
    const wrapper = mount(RichTextEditor, {
      attachTo: document.body,
      props: { modelValue: initialDocument },
    })

    await waitForEditor(wrapper)

    expect(wrapper.text()).toContain('初始内容')

    wrapper.unmount()
  })

  it('emits tiptap json after editing', async () => {
    const wrapper = mount(RichTextEditor, {
      attachTo: document.body,
      props: { modelValue: initialDocument },
    })
    await waitForEditor(wrapper)
    const surface = wrapper.find('.rich-text-content')

    surface.element.textContent = '更新内容'
    surface.element.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'insertText', data: '更新内容' }))
    await nextTick()

    const updates = wrapper.emitted('update:modelValue') ?? []
    const latest = updates.at(-1)?.[0]

    expect(typeof latest).toBe('string')
    expect(String(latest)).toContain('更新内容')

    wrapper.unmount()
  })

  it('updates the editor when model value changes', async () => {
    const wrapper = mount(RichTextEditor, {
      attachTo: document.body,
      props: { modelValue: initialDocument },
    })
    await waitForEditor(wrapper)
    const nextDocument = JSON.stringify({
      type: 'doc',
      content: [{ type: 'paragraph', content: [{ type: 'text', text: '切换内容' }] }],
    })

    await wrapper.setProps({ modelValue: nextDocument })
    await nextTick()

    expect(wrapper.text()).toContain('切换内容')

    wrapper.unmount()
  })

  it('keeps boundary whitespace when model value changes to plain text', async () => {
    const wrapper = mount(RichTextEditor, {
      attachTo: document.body,
      props: { modelValue: initialDocument },
    })
    await waitForEditor(wrapper)

    await wrapper.setProps({ modelValue: '  切换内容  ' })
    await nextTick()

    const text = wrapper.find('.rich-text-content').element.textContent
    expect(text).toBe('  切换内容  ')

    wrapper.unmount()
  })
})

async function waitForEditor(wrapper: ReturnType<typeof mount>) {
  for (let index = 0; index < 5; index += 1) {
    await nextTick()
    if (wrapper.find('.rich-text-content').exists()) return
  }
}
