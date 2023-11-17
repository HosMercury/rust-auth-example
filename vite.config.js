// import 'vite/modulepreload-polyfill';
// import path from 'path';
import { defineConfig } from 'vite';

export default defineConfig({
  build: {
    // assetsInlineLimit: 4096,
    manifest: true,
    rollupOptions: {
      input: './resources/js/main.js',
    },
  },
});
