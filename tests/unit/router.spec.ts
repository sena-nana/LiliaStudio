import { render, screen } from '@testing-library/vue'
import { createPinia } from 'pinia'
import { describe, expect, it } from 'vitest'
import App from '@/App.vue'
import { router } from '@/router'

async function renderAt(path: string) {
  await router.push(path)
  await router.isReady()
  return render(App, { global: { plugins: [createPinia(), router] } })
}

describe('router shell integration', () => {
  it('renders Ameya home inside template shell', async () => {
    const view = await renderAt('/')

    expect(await screen.findByText('Ameya')).toBeInTheDocument()
    expect(view.container.querySelector('.secondary-panel')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: '项目' })).toBeInTheDocument()
  })

  it('settings page uses settings sidebar and defaults to appearance', async () => {
    await renderAt('/settings')

    expect(await screen.findByRole('heading', { level: 1, name: '外观' })).toBeInTheDocument()
    expect(document.querySelector('.settings-sidebar__tabs')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /外观/ })).toHaveClass('is-active')
  })

  it('settings tab query can show about page', async () => {
    await renderAt('/settings?tab=about')

    expect(await screen.findByRole('heading', { level: 1, name: '关于' })).toBeInTheDocument()
    expect(await screen.findByText('Tauri 2 + Vue 3 + Pinia')).toBeInTheDocument()
  })

  it('unknown route redirects to home', async () => {
    await renderAt('/missing')

    expect(router.currentRoute.value.path).toBe('/')
  })
})
