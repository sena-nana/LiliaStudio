import { fireEvent, render, waitFor } from '@testing-library/vue'
import { createMemoryHistory, createRouter } from 'vue-router'
import { beforeEach, describe, expect, it } from 'vitest'
import { SIDEBAR_CONFIG } from '@/config/appShell'
import AppShell from '@/layouts/AppShell.vue'

async function renderAppShell(initialRoute = '/') {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      {
        path: '/',
        component: AppShell,
        meta: { sidebar: 'main', returnable: true },
        children: [
          { path: '', component: { template: '<div>home</div>' }, meta: { sidebar: 'main', returnable: true } },
          { path: 'settings', component: { template: '<div>settings</div>' }, meta: { sidebar: 'settings', lockSidebar: true, returnable: false } },
          { path: 'jobs', component: { template: '<div>jobs</div>' }, meta: { sidebar: 'main', returnable: true } },
        ],
      },
    ],
  })
  await router.push(initialRoute)
  await router.isReady()
  const view = render({ template: '<RouterView />' }, { global: { plugins: [router] } })
  return { ...view, router }
}

function shellElement(container: Element): HTMLElement {
  const shell = container.querySelector('.shell')
  if (!(shell instanceof HTMLElement)) throw new Error('shell not found')
  return shell
}

function leftResizer(container: Element): HTMLElement {
  const resizer = container.querySelector('.shell__resizer')
  if (!(resizer instanceof HTMLElement)) throw new Error('resizer not found')
  return resizer
}

beforeEach(() => {
  localStorage.clear()
})

describe('AppShell sidebar', () => {
  it('renders Ameya shell navigation and footer entries', async () => {
    const view = await renderAppShell()

    expect(view.getByText('Ameya')).toBeInTheDocument()
    expect(view.container.querySelector('.secondary-panel')).toBeInTheDocument()
    expect(view.getByRole('link', { name: 'Projects' })).toBeInTheDocument()
    expect(view.getByRole('link', { name: 'Library' })).toBeInTheDocument()
    expect(view.getByRole('link', { name: 'Settings' })).toBeInTheDocument()
    expect(view.getByRole('link', { name: 'Local-first workspace. Open the AI job queue.' })).toHaveClass('sb-conn--ok')
  })

  it('toggles sidebar collapse state and persists it', async () => {
    const view = await renderAppShell()
    const shell = shellElement(view.container)
    const collapse = view.container.querySelector('.titlebar__left-sidebar-btn')

    if (!(collapse instanceof HTMLElement)) throw new Error('toggle not found')
    expect(shell).not.toHaveClass('is-sidebar-collapsed')
    await fireEvent.click(collapse)

    expect(shell).toHaveClass('is-sidebar-collapsed')
    expect(leftResizer(view.container)).toHaveAttribute('aria-disabled', 'true')
    expect(localStorage.getItem(SIDEBAR_CONFIG.collapsedStorageKey)).toBe('1')
  })

  it('settings route uses settings sidebar and locks collapse toggle', async () => {
    localStorage.setItem(SIDEBAR_CONFIG.collapsedStorageKey, '1')
    const view = await renderAppShell('/settings')
    const shell = shellElement(view.container)
    const toggle = view.container.querySelector('.titlebar__left-sidebar-btn')

    expect(shell).toHaveClass('is-settings-mode')
    expect(shell).not.toHaveClass('is-sidebar-collapsed')
    expect(toggle).toBeDisabled()
    expect(view.container.querySelector('.settings-sidebar__tabs')).toBeInTheDocument()
    expect(view.container.querySelector('.sb-tree')).not.toBeInTheDocument()

    await fireEvent.click(view.getByRole('button', { name: /About/ }))
    await waitFor(() => {
      expect(view.router.currentRoute.value.fullPath).toBe('/settings?tab=about')
    })
  })
})
