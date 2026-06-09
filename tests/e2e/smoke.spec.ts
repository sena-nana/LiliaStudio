import { expect, test } from '@playwright/test'

test.setTimeout(90_000)

test('opens the template-based Ameya shell', async ({ page }) => {
  await page.goto('/', { waitUntil: 'domcontentloaded', timeout: 60_000 })

  await expect(page.getByText('Ameya')).toBeVisible({ timeout: 60_000 })
  await expect(page.getByRole('link', { name: 'Projects' })).toBeVisible()
  await expect(page.getByRole('link', { name: 'Library' })).toBeVisible()
  await expect(page.getByRole('link', { name: 'Settings' })).toBeVisible()
  await expect(page.locator('.shell__main')).toBeVisible()
})
