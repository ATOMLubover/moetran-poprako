import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// 默认使用 IPv4 回环地址，避免在某些 Windows 环境中对 ::1 的绑定导致 EACCES
const defaultHost = '127.0.0.1';

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [vue()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 5173,
    strictPort: true,
    host: host || defaultHost,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },
}));
