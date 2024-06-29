import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
  ],
  server: {
    proxy: {
      '^/game/[0-9]*/ws': {
        target: "ws://localhost:8061",
        ws: true
      },
      '/game': "http://localhost:8061",
      '/scenario': "http://localhost:8061",
      '/login': "http://localhost:8061",
    }
  },
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  }
})
