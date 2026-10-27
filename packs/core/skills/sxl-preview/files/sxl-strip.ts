import type { Plugin } from 'vite';

// Strip sixloom trace attributes from production bundles. Works on any framework's
// compiled output: object props ("data-sxl": "…" from React/Vue/Solid) and HTML
// template strings (data-sxl="…" from Svelte/Astro/Lit). Dev and preview keep them.
export function sxlStrip(): Plugin {
  return {
    name: 'sxl-strip',
    apply: 'build',
    renderChunk(code) {
      const out = code
        .replace(/["']data-sxl["']\s*:\s*(["'`])[^"'`]*\1\s*,?/g, '')
        .replace(/\sdata-sxl=(\\?["'])[^"'\\]*\1/g, '');
      return out === code ? null : { code: out, map: null };
    },
  };
}
