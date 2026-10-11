import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [svelte()],
  server: {
    port: 5183,
    strictPort: true
  },
  preview: {
    port: 5173
  },
  build: {
    outDir: '../src-tauri/../ui/build',
    emptyOutDir: true,
    sourcemap: true
  }
});