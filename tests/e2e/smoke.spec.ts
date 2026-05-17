import { expect, test } from '@playwright/test'

test('opens the new object-workspace shell', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByRole('link', { name: /项目/ })).toBeVisible()
  await expect(page.getByRole('link', { name: /资料/ })).toBeVisible()
  await expect(page.getByText('Inspector')).toBeVisible()
  await expect(page.getByText('任务空闲')).toBeVisible()
})
