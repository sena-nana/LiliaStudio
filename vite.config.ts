import { fileURLToPath, URL } from 'node:url'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'

const host = process.env.TAURI_DEV_HOST
const ameyaDevPort = Number.parseInt(process.env.AMEYA_DEV_PORT ?? '', 10)
const strictPort = process.env.AMEYA_DEV_STRICT_PORT === '1'
const port = Number.isInteger(ameyaDevPort) ? ameyaDevPort : 1420

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  clearScreen: false,
  server: {
    strictPort: strictPort || port === 1420,
    port,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  envPrefix: ['VITE_', 'TAURI_'],
})
