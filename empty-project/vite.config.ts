import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    // Bind on all interfaces (IPv4 + IPv6) so http://localhost, http://127.0.0.1
    // and access from other devices all work; pin the port for a stable URL.
    host: true,
    port: 5173,
    strictPort: true,
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
  },
});
