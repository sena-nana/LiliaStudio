import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import { createPinia } from 'pinia'
import App from '@/App.vue'
import { router } from '@/router'

describe('AppShell', () => {
  it('renders workspace tabs, collection rail, context panel, and status bar', async () => {
    await router.push('/')
    await router.isReady()

    const wrapper = mount(App, {
      global: {
        plugins: [createPinia(), router],
      },
    })

    expect(wrapper.text()).toContain('Ameya')
    expect(wrapper.find('nav[aria-label="对象标签"]').text()).toContain('项目')
    expect(wrapper.find('nav[aria-label="对象标签"]').text()).toContain('资料')
    expect(wrapper.find('aside[aria-label="集合栏"]').exists()).toBe(true)
    expect(wrapper.find('aside[aria-label="上下文面板"]').exists()).toBe(true)
    expect(wrapper.find('footer.status-bar').text()).toContain('任务空闲')

    wrapper.unmount()
  })
})
