import { defineConfig, type Plugin } from 'vite';
import react from '@vitejs/plugin-react';

// Strip sixloom trace attributes (data-sxl) from production bundles.
function sxlStrip(): Plugin {
  return {
    name: 'sxl-strip',
    apply: 'build',
    renderChunk(code) {
      return { code: code.replace(/["']data-sxl["']\s*:\s*(["'`])[^"'`]*\1\s*,?/g, ''), map: null };
    },
  };
}

export default defineConfig({
  plugins: [react(), sxlStrip()],
  server: { proxy: { '/api': process.env.API_URL ?? 'http://localhost:8787' } },
});
