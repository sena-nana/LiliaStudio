import { expect, test } from '@playwright/test'

test('opens Ameya shell', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByText('Ameya')).toBeVisible()
  await expect(page.getByText('选择或创建一个创作宇宙')).toBeVisible()
})
