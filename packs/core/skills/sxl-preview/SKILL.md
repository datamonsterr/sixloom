---
name: sxl-preview
description: The sixloom preview loom. It builds a clickable preview of every screen and state in design/SCREENS.md, in the project's real front-end framework, running on mock data with realistic latency and errors (MSW), so humans can review the product before any backend exists. The preview becomes the production front end. Use after the design loom, when the user says "build the preview", "front end", "/sxl-preview", or wants a clickable prototype or mock UI.
---

# sxl-preview: the preview loom

Follow the **sxl-loom** protocol. Loom ID: `preview`. Run it in a new
session. The context pack is `spec/SPEC.md`, `design/SCREENS.md`,
`design/DESIGN.md` and `design/exports/`.

The framework comes from `sixloom.toml` (`frontend.framework`). Load the
matching skill before writing code: React + Vite → `vercel-react-best-practices`,
`vercel-composition-patterns`; Next.js → the same two; Vue/Nuxt → `vue`, `nuxt`;
SvelteKit → `svelte-code-writer`, `svelte-core-bestpractices`. Add
`shadcn-ui` when `frontend.ui_kit = "shadcn"`.

## Architecture: built to become the real app

```
apps/web/
  src/api/            # typed API client: the ONLY place that talks to the network
  src/mocks/
    handlers.ts       # MSW handlers, one per endpoint the UI needs
    fixtures/         # realistic data, typed with the client's types
    scenarios.ts      # named states: ready, empty, error, slow, partial, …
    browser.ts        # starts the MSW worker when VITE_API_MODE=mock
  src/routes|pages/   # one route per screen ID in SCREENS.md
```

Rules:

1. Components never call `fetch` directly. They call `src/api`. Switching to
   the real backend later is one environment variable (`*_API_MODE=real`) and
   nothing else.
2. The client's types are named after the spec domain. When
   `contracts/openapi.yaml` exists, generate the types from it
   (`openapi-typescript`) and do not hand-write them.
3. Every element that a test will touch gets a stable
   `data-testid="<screen-id>.<control>"`, for example `S-02.new-project`.
4. **Trace without touching behavior**: the element that delivers an AC gets
   `data-sxl="F-003.AC1"` (several IDs are space-separated), so a reviewer can
   inspect any element in devtools and paste the ID into the docs search or
   `sixloom trace`. Strip `data-sxl` from production builds with a build
   transform: for React, `babel-plugin-react-remove-properties` with
   `properties: ["data-sxl"]` (or an SWC or esbuild equivalent); for
   Vue, Svelte or Astro, a compile-time node transform. Keep it in dev and
   preview mode.

## Mock behavior: make the states reviewable

- **Latency**: each handler waits for a realistic delay (`delay('real')`, or
  a per-endpoint value such as list 400–900ms or save 300ms), so skeletons
  actually show.
- **Scenarios**: every state in `SCREENS.md` is reachable on demand with a
  URL parameter, `?sxl=<scenario>` (for example `?sxl=empty` or
  `?sxl=error`), or with a small dev-only toolbar for switching the scenario
  per screen. See [MOCKS.md](MOCKS.md) for the pattern.
- **User errors**: validation errors come from the mock exactly as the API
  will return them (status 422 and a field error map), so the form error UI
  is real.
- **System errors**: 500, timeout, offline and 403 scenarios on every screen
  that loads data.
- **Statefulness**: create, update and delete operations change in-memory
  fixtures for the session, so flows feel real (create a project, then see
  it in the list).

## Build order

1. Scaffold the app (framework skill), add the tokens from `DESIGN.md` as
   CSS variables, and add the layout shell and routing for every screen ID.
2. Add the client and the mock layer, then the screens in flow order, each
   with all of its states.
3. Write `apps/web/PREVIEW.md`: how to run it (`just preview`) and a table
   of every screen × state with its direct URL, so the reviewer can click
   through the whole matrix.
4. Smoke-check every URL in that table with Playwright (`playwright-cli`
   skill). Each one must render without console errors. Take a screenshot of
   each one to `apps/web/.preview-shots/` (gitignored) for the visual check
   during validation.

## Sync rule: the preview and the spec move together

This loom is where the product gets finished, because the user reviews it by
clicking. Expect several rounds of "move this", "add that", "this state is
wrong". Each round that changes behavior **updates `spec/SPEC.md` in the same
round**: edit the AC text, add the AC, or record a removal, and note it in a
`sync` change record (`sixloom change new "<what>" --kind sync`).
Pure visual tweaks (spacing, color) only update `DESIGN.md`. The gate does not
pass while the preview shows behavior the spec does not describe.

## Gate (product reviewer)

Give the reviewer the `PREVIEW.md` URL table, then ask:

1. Clicking through each key flow, does it work the way you expected?
2. Do the loading, empty and error states look right on every screen?
3. Do the form errors read well and keep what you typed?
4. Does anything feel slow, confusing or missing?
5. Are the listed sync differences (if any) resolved the way you want?

## Next

The engineering track starts from this approved preview: `sixloom data`
(the sxl-data-model loom) in a new session. Its client types and mock
fixtures are the starting point for the data model and the API contract.
