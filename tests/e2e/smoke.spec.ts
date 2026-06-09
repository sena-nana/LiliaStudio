import { expect, test } from '@playwright/test'

test.setTimeout(90_000)

test('opens the template-based Ameya shell', async ({ page }) => {
  await page.goto('/', { waitUntil: 'domcontentloaded', timeout: 60_000 })

  await expect(page.getByText('Ameya')).toBeVisible({ timeout: 60_000 })
  await expect(page.getByRole('link', { name: '项目' })).toBeVisible()
  await expect(page.getByRole('link', { name: '资料库' })).toBeVisible()
  await expect(page.getByRole('link', { name: '设置' })).toBeVisible()
  await expect(page.locator('.shell__main')).toBeVisible()
})
