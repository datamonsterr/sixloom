# The sixloom framework

This document describes the framework and records the design review behind
it: the weak points found, and the decision taken for each one. The
[README](../README.md) is the how-to; this file is the why.

Decision status: **confirmed** means the team agreed to it in the design
review; **default** means it stands until someone challenges it with a change
record.

---

## 1. What sixloom is

sixloom turns an already-discovered solution into a working product with
coding agents. It optimizes three things together:

1. The product **looks and works as expected**.
2. **Total development time** is short.
3. **Total tokens spent** are few.

It assumes that problem discovery is done: the team arrives with user stories
or use cases and an initial idea of the solution.

### Mindset

- **Small, fast feedback loops.** Each step is a short loop of
  agent drafts → human reacts → agent revises.
- **Humans review only what is easy for humans to review**: a short list of
  features, a clickable front end, a database diagram, an API reference. Never
  a hundred files of code.
- **Human attention goes into the goal.** People clarify what to build and
  review the result *before it exists*, because the preview and the docs are
  the result, seen early. After that, the code is written by agents in a
  test-driven loop.

### The phases

Each phase is a **loom**: one agent session that produces one artifact,
iterated with the human until it is right, then saved and committed by a
`sixloom` command. Each loom has a skill that starts it, runs the loop, shows
a short gate checklist, and ends it.

**Phase 1: the product track (humans are present).**

1. **Spec** (`sxl-specify`). From `init`, the agent writes `spec/SPEC.md`: a
   short, concise list of features. Each feature has acceptance criteria,
   built from the user stories and use cases of discovery. It holds only what
   a human must understand and confirm.
2. **UI design** (`sxl-design`). In a new session with the spec (and the
   team's design system, if any), the agent produces `design/SCREENS.md`
   (every screen, every functional control, every state including loading
   skeleton and error, and the user flows), `design/DESIGN.md` (the design
   system), and visual comps of the key screens. Comps are made in Claude
   Design (saved as `.dc.html` files) or in Google Stitch (through the
   `stitch` CLI, pulled back into the repo as HTML).
3. **Preview** (`sxl-preview`). The agent builds every screen and state in
   the team's real front-end framework, on mock data with realistic latency,
   loading skeletons, and user and system errors. The preview **is** the
   real front end. Later, only the backend is added behind it. This is where
   the product gets finished: the user clicks, asks for changes, and every
   change that alters behavior updates the spec in the same round.

**Phase 1, continued: the engineering track (starts from the final preview).**

4. **Data model** (`sxl-data-model`). This is data-first design: entities,
   relationships, constraints and lifecycles, derived from the spec and from
   the preview's actual data needs. It is a docs page with a rendered
   database diagram.
5. **System design** (`sxl-system-design`). The high-level picture: services,
   stores and integrations, key request flows, and stack decisions as ADRs.
   Mermaid is the source; a draw.io file is the editable view.
6. **API and docs** (`sxl-api-docs`). The OpenAPI contract is derived from
   the preview's API client and mocks. The docs site (Fumadocs, Docusaurus or
   Starlight) holds everything a newcomer reads: the API reference generated
   from the contract, architecture, install guide, codebase tour and
   changelog.

**Phase 2: the autonomous build (agents work; humans wait for the result).**

7. **Test suites** (`sxl-test-suites`). Before implementation, the agent
   writes one Playwright e2e test per acceptance criterion and proves each
   one green against the preview. It also writes contract and integration
   tests per API operation, red until implemented, plus a backend skeleton,
   a `justfile`, and fast, cached, parallel GitHub Actions CI. The acceptance
   tests are then **locked**.
8. **Build** (`sxl-build`). With "the preview plus the API docs" as the goal,
   agents implement the backend in vertical slices with TDD (unit tests are
   written inside the loop). Subagents work in parallel in separate
   worktrees, with one PR per slice; each waits for CI to go green and then
   merges.
9. **Validate** (`sxl-validate`). The agent checks the working product
   against the goal, looking for gaps and bugs, fixes them, and reports. The
   user then decides whether it is done.

**After that: feedback → patch → changelog stack.** Feedback on the working
product becomes a change record (`sxl-patch`). It updates the spec first,
then the affected artifacts, then the locked tests, and only then the code.
The records form the changelog stack, which feeds the next cycle.

### How the user drives it

The user stays in their coding agent (Claude Code, Codex, OpenCode, Gemini
CLI, Kimi, …) and talks: "sixloom init", "sixloom spec", "sixloom preview",
"approved", "sixloom next". The `sixloom` skill maps each request to a CLI
call and a loom skill, and the agent runs the CLI itself. The terminal UI
(`sixloom` with no arguments) is there for people who want a dashboard, but
nobody has to use it.

---

## 2. Design review: weak points and decisions

### W1. Nothing tied the artifacts together

**D-01 (confirmed): stable IDs, kept out of the product.** Every feature has
an ID (`F-001`), and every acceptance criterion has one (`F-001.AC2`). The IDs
never appear in API paths, payloads or application logic. They appear only as
markers:

| Where | Marker | Found by |
|---|---|---|
| Docs pages (screens, data model, system design) | `<sub>sxl: F-003.AC1</sub>` under the relevant heading | the docs site search |
| OpenAPI operation descriptions | last line `sxl: F-003.AC1` | the API reference and its search |
| Front end | `data-sxl="F-003.AC1"` (stripped from production builds) | browser devtools |
| Tests | the title starts with the AC ID | test reports |

A reviewer can inspect an element in the preview, copy its ID, paste it into
the docs search, and land on the endpoint and the entity behind it.
`sixloom trace` joins all the markers into one matrix and lists the gaps.

### W2. The two tracks could contradict each other

**D-02 (confirmed): the product track finishes first.** The preview is
iterated until it is final, and every behavior change to it updates the
spec. The engineering track then derives the data model and the API contract
from the approved preview (its client types and mock fixtures), so there is
nothing to reconcile after the fact. The API loom still validates the mocks
against the contract, and a mismatch means that the contract is wrong.

**D-03 (confirmed): the preview is the real front end.** Components reach
the network only through `src/api/`, which runs on MSW mocks in preview mode.
The build switches one environment variable, and the same screens hit the
real backend.

### W3. UI design had no reviewable artifact

**D-04 (confirmed).** The design loom always writes `design/DESIGN.md` and
`design/SCREENS.md`. Comps live in the repo: Claude Design comps as `.dc.html`
files, and Stitch comps pulled back as HTML through the `stitch` CLI (the user
signs in once with `stitch login`), with the Stitch IDs recorded in
`design/exports/stitch.json`. `design.fidelity = "key"` limits comps to the
3–5 screens that set the visual language.

### W4. "Easy to review" needed a definition

**D-05 (confirmed).** Each loom ends with a gate of five yes/no questions or
fewer, written for its reviewer: product for the spec, design, preview and
validate; engineering for the data model, system design and API. Anything a
tool can check is checked by `sixloom loom check` or `sixloom loom end`, and
never put in front of the human.

### W5. Writing every test first is horizontal slicing

**D-06 (confirmed): test tiers, each with its own timing.** E2E tests (one
per AC) are written before the build and proven green on the preview, so they
check agreed behavior, not imagined behavior. Contract and integration tests
are written before the build and are red until implemented. Unit tests are
written only inside the TDD loop.

**D-07 (confirmed): strict test rules.** `sixloom tests lock` fingerprints
every locked test's title and its ordered assertion matchers with their
arguments. `sixloom tests verify`, which runs in CI, fails if a locked test
is removed, renamed, skipped (`skip`, `only`, `fixme`) or has an assertion
changed. Selector and wait repairs keep the fingerprint, so they are allowed.
A locked test that seems wrong is raised as a `test` change record, never
edited.

### W6. Parallel agents, PRs and CI were fragile

**D-08 (confirmed): vertical slices in dependency order.** A slice is one
feature. The order comes from the data model, and slices with disjoint write
sets run in parallel (`build.max_parallel`, default 3), each in its own git
worktree and branch, with one PR per slice and serial merges. Without
subagents, slices run one after another; without GitHub, they merge locally
after `just ci`. `build.max_attempts_per_slice` (default 3) stops a stuck
slice: it is marked blocked with a diagnosis, and the build moves on.

**D-09 (confirmed): CI calls the same targets as local.** Every workflow step
is a `just` target. Jobs are path-filtered, lint, typecheck and unit run in
parallel, e2e is sharded 4 ways, caches are keyed by lockfile, superseded runs
are cancelled, and a failing job prints the `just` command that reproduces it.

### W7. Every new session paid to rediscover the project

**D-10 (confirmed): context packs.** `sixloom loom start <loom>` writes
`.sixloom/context/<loom>.xml`, which holds the loom's declared inputs and
nothing else, in one file with per-file token counts. The layout follows
[repomix](https://github.com/yamadashy/repomix) (`<file_summary>`,
`<directory_structure>`, `<files>` with `<file path="…">` entries), a widely
used format for packing repository content for LLMs. A new session reads one
file instead of exploring the repo.

**D-11 (confirmed): stale detection.** Approval records a hash of each input
path. When an input changes, `sixloom status` marks every loom that depends
on it `stale` and names the changed input, so the next session updates only
that part.

### W8. The changelog stack was undefined

**D-12 (default).** A change record is `spec/changes/CH-0007-<slug>.md`, with a
kind (`feedback`, `bug`, `sync` or `test`), a status
(`proposed → accepted → specced → tested → built`, or `rejected`), the IDs it
affects, and the looms it invalidates. `SPEC.md` is always the current truth.
The records explain how it got there, and are applied oldest first.
`spec/CHANGELOG.md` is generated from them.

### W9. "Done" was not measurable

**D-13 (confirmed): machine conditions, then the user's sign-off.**
`sixloom done` checks:

1. every loom is approved and none is stale;
2. the locked tests are intact;
3. the trace is complete (every AC is on a screen, and every `must` AC is tested);
4. the changelog stack is empty (every change is built or rejected);
5. the latest validate report says `result: pass`. That report covers the
   locked suite in real mode, preview-versus-real screenshots of every screen
   state within tolerance, docs truth, no console errors, no serious axe
   violations, CI green on main, no blocked slices, and an exploratory pass.

It then lists what only a human can confirm. The product is done only when
the user says so and the agent runs `sixloom done --confirm`, which records
their sign-off.

### W10. Diagrams cost tokens and are hard to diff

**D-14 (default).** Mermaid is the source of every diagram, because it
renders on GitHub and in all three docs engines. `docs/diagrams/system.drawio`
is generated from it for people who want to edit visually.

### W11. Existing projects

**D-15 (default).** `sixloom init --adopt` adds the framework to an existing
repo without touching application code; `sxl-specify --from-code` drafts a
spec from the code, and the spec is marked reverse-engineered.

### W12. Harness differences

**D-16 (default).** Skills plus the `sixloom` CLI are the only required
surface. Subagents, hooks and plugins are optional speed-ups, and every skill
states its fallback.

### W13. Too many skills in context

**D-17 (default).** `sixloom init` installs only the skills for the chosen
front end, docs engine and design tool. With `--bank`, the rest go into an
[other-skill](https://github.com/datamonsterr/other-skill) bank
(`.otherskill/bank/<category>/…`), where a router injects them only when a
prompt needs them. Pack IDs are other-skill category IDs. Teams can always
add their own skills with `sixloom skills add`.

---

### W14. The folder layout assumed one web app and one API

`apps/web` and `apps/api` were hard-coded, but an interface can be a TUI, a
CLI, a mobile or desktop app, there can be several, and the number of services
and their pattern depend on the system design.

**D-18 (confirmed): the layout is configuration, not convention.**
`sixloom.toml` records:

- **`[[interfaces]]`**, chosen at `init`: kind (`web`, `tui`, `cli`,
  `mobile`, `desktop`), framework, folder, API client module, and mocks.
- **`[system] pattern`** and **`[[services]]`** (kind, stack, folder),
  recorded by the system-design loom with `sixloom layout`.
- **`[tests] locked`**: the folders of the locked acceptance tests.

Every loom resolves its context pack, outputs, checks and stale detection
from these. So do the trace (`data-sxl` on the web, an `sxl:` comment
elsewhere) and the test lock, which reads JS/TS, Rust, Python and Go tests,
with AC IDs in titles or function names. The rules block in
`AGENTS.md`/`CLAUDE.md` is regenerated whenever the layout changes, so agents
always see the current folders. `sixloom loom check system` fails until the
pattern and services are recorded.

## 3. Choices for v0.1

| Area | Choice | Why |
|---|---|---|
| Docs engines | **Fumadocs** (default), **Docusaurus**, **Starlight** | MDX, Mermaid, and an OpenAPI reference generated from the contract in all three. Fumadocs is closest to the GitBook feel, Docusaurus is the most widely used, and Starlight is the lightest. |
| Interfaces | web (React + Vite default, Next.js, Vue, Nuxt, SvelteKit), TUI (Ratatui, Bubble Tea, Textual, Ink), CLI (clap, Cobra, Typer, Commander), mobile (Expo, Flutter), desktop (Tauri, Electron) | Web and Expo are backed by vetted upstream skills; the others follow the generic kind patterns in `sxl-preview`. |
| Design | Claude Design (default), Google Stitch, none | Stitch through Google's official `stitch` CLI skill. |
| Mocks | MSW | The same handlers serve the browser, Node tests and Playwright. |
| E2E | Playwright | Parallel and sharded, with Microsoft's `playwright-cli` skill. |
| Tasks | `just` | One command surface for humans, agents and CI. |
| CLI | Rust + ratatui | One static binary that embeds every skill and template. |

## 4. Open questions

These have defaults until a human decides.

1. **Backend stack**: stay agnostic, with the system loom recording the
   choice as an ADR? (Default: agnostic.)
2. **Who approves gates**: any committer, or product for the product track
   and engineering for the engineering track? (Default: any committer;
   approval records the git author.)
3. **Slice merges**: auto-merge on green CI? (Default: yes; the final
   validate sign-off is human.)
4. **Visual tolerance** for preview versus real. (Default: 0.5% of pixels per
   screen state.)
5. **Docs hosting**: should init also set up GitHub Pages or Vercel?
   (Default: no; local `just docs` only.)
