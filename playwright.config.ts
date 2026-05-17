import { defineConfig } from '@playwright/test'

const e2ePort = Number(process.env.E2E_PORT ?? 1430)
const baseURL = `http://127.0.0.1:${e2ePort}`

export default defineConfig({
  testDir: 'tests/e2e',
  webServer: {
    command: `pnpm exec vite --host 127.0.0.1 --port ${e2ePort} --strictPort`,
    url: baseURL,
    reuseExistingServer: false,
    timeout: 120_000,
  },
  use: {
    baseURL,
  },
})
