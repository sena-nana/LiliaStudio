import { describe, expect, it, vi } from 'vitest'

describe('useTheme', () => {
  it('restores theme from localStorage and writes html data-theme', async () => {
    localStorage.setItem('ameya.theme', 'light')
    vi.resetModules()

    const { useTheme } = await import('@/composables/useTheme')
    const { theme } = useTheme()

    expect(theme.value).toBe('light')
    expect(document.documentElement.dataset.theme).toBe('light')
  })

  it('setTheme syncs data-theme and localStorage', async () => {
    vi.resetModules()
    const { useTheme } = await import('@/composables/useTheme')
    const { theme, setTheme } = useTheme()

    setTheme('dark')

    expect(theme.value).toBe('dark')
    expect(document.documentElement.dataset.theme).toBe('dark')
    expect(localStorage.getItem('ameya.theme')).toBe('dark')
  })
})
