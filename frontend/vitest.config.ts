/**
 * Vitest configuration for Steel Pipe DB frontend.
 *
 * Uses jsdom for DOM simulation (no real browser needed).
 * Path alias `@/` → `src/` mirrors the Vite config.
 * setup.ts runs before each test to configure @testing-library/jest-dom matchers.
 */
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
    css: false,
  },
});
