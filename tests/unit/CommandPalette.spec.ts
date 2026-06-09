import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import { nextTick } from 'vue'
import CommandPalette from '@/components/command/CommandPalette.vue'
import { router } from '@/router'

describe('CommandPalette', () => {
  it('opens with shell navigation and app menu entries', async () => {
    await router.push('/')
    await router.isReady()

    const wrapper = mount(CommandPalette, {
      attachTo: document.body,
      global: {
        plugins: [router],
      },
    })

    try {
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'k', ctrlKey: true }))
      await nextTick()

      expect(wrapper.find('[role="dialog"]').exists()).toBe(true)
      expect(wrapper.find('ul').exists()).toBe(true)
      expect(wrapper.findAll('li').length).toBeGreaterThanOrEqual(12)
      expect(document.activeElement).toBe(wrapper.find('button').element)
      expect(wrapper.text()).toContain('Projects')
      expect(wrapper.text()).toContain('Library')
      expect(wrapper.text()).toContain('Settings')
      expect(wrapper.text()).toContain('Prompt templates')

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
      await nextTick()

      expect(wrapper.find('[role="dialog"]').exists()).toBe(false)
    } finally {
      wrapper.unmount()
    }
  })
})
