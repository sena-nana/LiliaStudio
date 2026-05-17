import { expect, test } from '@playwright/test'

test.setTimeout(90_000)

test('opens the new object-workspace shell', async ({ page }) => {
  await page.goto('/', { waitUntil: 'domcontentloaded', timeout: 60_000 })
  await expect(page.getByRole('navigation', { name: '对象标签' }).getByRole('link', { name: '项目' })).toBeVisible({
    timeout: 60_000,
  })
  await expect(page.getByRole('navigation', { name: '对象标签' }).getByRole('link', { name: '资料' })).toBeVisible()
  await expect(page.getByRole('complementary', { name: '集合栏' })).toBeVisible()
  await expect(page.getByRole('complementary', { name: '上下文面板' })).toBeVisible()
  await expect(page.getByText('任务空闲')).toBeVisible()
})
