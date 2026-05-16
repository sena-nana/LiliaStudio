import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import { createPinia } from 'pinia'
import App from '@/App.vue'
import { router } from '@/router'

describe('AppShell', () => {
  it('renders the primary navigation and tool layout', async () => {
    router.push('/')
    await router.isReady()

    const wrapper = mount(App, {
      global: {
        plugins: [createPinia(), router],
      },
    })

    expect(wrapper.text()).toContain('Ameya')
    expect(wrapper.text()).toContain('项目库')
    expect(wrapper.text()).toContain('上下文')
  })
})
