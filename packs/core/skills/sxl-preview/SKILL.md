---
name: sxl-preview
description: The sixloom preview loom. It builds every screen and state in design/SCREENS.md in each of the project's interfaces (web, TUI, CLI, mobile or desktop), in the real framework, running on mock data with realistic latency and errors, so humans can review the product before any backend exists. The preview becomes the production interface. Use after the design loom, when the user says "build the preview", "front end", "TUI", "/sxl-preview", or wants a clickable prototype or mock UI.
---

# sxl-preview: the preview loom

Follow the **sxl-loom** protocol. Loom ID: `preview`. Run it in a new
session. The context pack is `spec/SPEC.md`, `design/SCREENS.md`,
`design/DESIGN.md` and `design/exports/`.

## Where it goes: read the layout, never assume it

Run `sixloom layout --json`. Each entry in `interfaces` gives you the `kind`,
the `framework`, the `path` (the interface's folder), `api_client` (the only
module that may talk to the backend), `mocks` (mock data and scenarios) and
`preview_doc` (the reviewer's guide). Build each interface exactly there. If
the folder layout the framework wants differs, for example a Rust crate whose
client lives in `src/client.rs`, record it first:

```bash
sixloom layout interface remove tui
sixloom layout interface add tui:ratatui:crates/tui --api-client src/client.rs --mocks src/mocks
```

That updates `sixloom.toml`, the agents' rules block and the installed skills.
Every later loom reads the paths from there.

Load the framework skill before writing code: React/Next.js →
`vercel-react-best-practices`, `vercel-composition-patterns`; Vue/Nuxt → `vue`,
`nuxt`; SvelteKit → `svelte-code-writer`, `svelte-core-bestpractices`; Expo →
`vercel-react-native-skills`; `shadcn-ui` when the interface's `ui_kit` is
`shadcn`. For other frameworks (Ratatui, Bubble Tea, Textual, Ink, clap,
Tauri, …) use the framework's own docs.

## The same architecture in every kind

```
<interface path>/
  <api_client>     the ONLY code that talks to the backend: one typed function per operation
  <mocks>/         in-memory fixtures + named scenarios, behind the same interface as the client
  screens…         one screen/view/command per screen ID in SCREENS.md
  PREVIEW.md       every screen × state and exactly how to reach it
```

1. Screens never touch the network. They call the API client. **Mock mode is
   one switch** (an env var or a flag), so the preview becomes the real
   interface when the backend exists, with no second implementation.
2. Domain types are named after the spec. Once `contracts/openapi.yaml`
   exists, they are generated from it.
3. **Every state in `SCREENS.md` is reachable on demand** through a named
   scenario (`ready`, `empty`, `error`, `slow`, `offline`, `forbidden`, …),
   with realistic latency so loading states actually show. Validation errors
   come from the mock exactly as the API will return them.
4. **Stable handles for tests**: every control a test touches has a stable
   name: `data-testid="S-02.new-project"` on the web, a widget or key ID in a
   TUI, a command and flag name in a CLI, an accessibility ID on mobile.
5. **Trace without touching behavior**: the element or code that delivers an
   AC carries its ID, so a reviewer can find it and paste it into the docs
   search or `sixloom trace`. On the web this is a `data-sxl="F-003.AC1"`
   attribute, stripped from production builds. Everywhere else it is an
   `// sxl: F-003.AC1` comment (or `#` in Python) on the screen's render or
   handler function. Comments never reach the binary.
6. **Realistic fixtures**: give IDs, dates and emails their real formats
   (UUIDs, ISO timestamps), because the API loom validates every fixture
   against the contract.

Kind-specific patterns:

- **web**: [WEB.md](WEB.md) covers MSW mocks, `?sxl=<scenario>`, and the
  `data-sxl` strip plugin.
- **tui, cli, mobile, desktop**: [NATIVE.md](NATIVE.md) covers a mock adapter
  behind the client trait, `SXL_SCENARIO`, and how reviewers run each state.

## Build order

1. Scaffold each interface in its folder (framework skill), apply the
   `DESIGN.md` tokens (CSS variables on the web; a theme or style module in a
   TUI or native app), and add navigation for every screen ID.
2. Add the client and the mock layer, then build the screens in flow order,
   each with all of its states.
3. Write the interface's `PREVIEW.md`: how to start it in preview mode
   (`just preview` or `just preview-<id>`) and a table of every screen × state
   with the exact URL, command or key sequence that reaches it.
4. Smoke-check every row of that table:
   - **web**: drive it with Playwright (`playwright-cli`), with no console
     errors, and screenshot each state to `<path>/.preview-shots/` (gitignored).
   - **TUI**: render each state in the framework's test backend, or run the
     binary in a pseudo-terminal, and save the text screen to
     `<path>/.preview-shots/<screen>.<state>.txt`.
   - **CLI**: run each command and save stdout and the exit code.

## Sync rule: the preview and the spec move together

This loom is where the product gets finished, because the user reviews it by
using it. Expect several rounds of "move this", "add that", "this state is
wrong". Each round that changes behavior **updates `spec/SPEC.md` in the same
round**: edit the AC text, add the AC, or record a removal, and note it in a
`sync` change record (`sixloom change new "<what>" --kind sync`). Pure visual
tweaks only update `DESIGN.md`. The gate does not pass while the preview shows
behavior the spec does not describe.

## Gate (product reviewer)

Give the reviewer each `PREVIEW.md`, then ask:

1. Going through each key flow, does it work the way you expected?
2. Do the loading, empty and error states look right on every screen?
3. Do input errors read well and keep what you typed?
4. Does anything feel slow, confusing or missing?
5. Are the listed sync differences (if any) resolved the way you want?

## Next

The engineering track starts from this approved preview: `sixloom data`
(the sxl-data-model loom) in a new session. Each interface's client types and
mock fixtures are the starting point for the data model and the API contract.
