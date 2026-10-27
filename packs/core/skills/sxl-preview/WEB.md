# Web interfaces (React, Next.js, Vue, Nuxt, SvelteKit, …)

The paths below are relative to the interface's `path` from `sixloom layout`.
The defaults are `api_client = src/api` and `mocks = src/mocks`.

```
src/api/            typed API client: the ONLY place that calls fetch
src/mocks/
  handlers.ts       MSW handlers, one per operation the UI needs
  fixtures/         realistic data, typed with the client's types
  scenarios.ts      named states read from ?sxl=<scenario>
  browser.ts        starts the MSW worker when VITE_API_MODE=mock
src/routes|pages/   one route per screen ID
PREVIEW.md          every screen × state with its URL
```

- **Mock mode**: put `VITE_API_MODE=mock` in `.env.mock` and make the `preview`
  script `vite --mode mock` (Next.js and Nuxt: start the worker in a
  client-only entry). The full MSW pattern, including scenarios, latency and
  the error envelope, is in [MOCKS.md](MOCKS.md). MSW 3 renamed
  `onUnhandledRequest` to `onUnhandledFrame`.
- **Scenarios**: `?sxl=empty`, `?sxl=error`, `?sxl=slow`, …, optionally with a
  dev-only toolbar to switch them per screen.
- **Test handles**: `data-testid="<screen-id>.<control>"`.
- **Trace**: `data-sxl="F-003.AC1"` (several IDs are space-separated). Strip
  it from production builds with [files/sxl-strip.ts](files/sxl-strip.ts), a
  framework-agnostic Vite plugin (`plugins: [react(), sxlStrip()]`) that
  handles React/Vue object props and Svelte/Astro HTML strings. After
  `vite build`, `grep -c data-sxl dist/assets/*.js` must print 0.
- **PREVIEW.md** lists one URL per screen × state, for example
  `http://localhost:5173/?sxl=empty`.
