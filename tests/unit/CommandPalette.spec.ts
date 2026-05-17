import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import { nextTick } from 'vue'
import CommandPalette from '@/components/command/CommandPalette.vue'
import { router } from '@/router'

describe('CommandPalette', () => {
  it('opens with workspace tabs and app menu entries', async () => {
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

      expect(wrapper.get('[role="dialog"]').attributes('aria-label')).toBe('命令面板')
      expect(wrapper.find('ul').exists()).toBe(true)
      expect(wrapper.findAll('li')).toHaveLength(12)
      expect(document.activeElement).toBe(wrapper.find('button').element)
      expect(wrapper.text()).toContain('项目')
      expect(wrapper.text()).toContain('资料')
      expect(wrapper.text()).toContain('设置')
      expect(wrapper.text()).toContain('Prompt 模板')

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
      await nextTick()

      expect(wrapper.find('[role="dialog"]').exists()).toBe(false)
    } finally {
      wrapper.unmount()
    }
  })
})
