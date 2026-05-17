import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import { createPinia } from 'pinia'
import App from '@/App.vue'
import { router } from '@/router'

describe('AppShell', () => {
  it('renders the workspace tabs, collection rail, and inspector scaffold', async () => {
    await router.push('/')
    await router.isReady()

    const wrapper = mount(App, {
      global: {
        plugins: [createPinia(), router],
      },
    })

    expect(wrapper.text()).toContain('Ameya')
    expect(wrapper.text()).toContain('项目')
    expect(wrapper.text()).toContain('资料')
    expect(wrapper.text()).toContain('Inspector')
    expect(wrapper.text()).toContain('任务空闲')

    wrapper.unmount()
  })
})
