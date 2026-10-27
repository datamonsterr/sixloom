# Dogfood report: v0.1

sixloom was tested by building a real project with it from an empty
directory to sign-off, then running one feedback cycle. The project is
[`examples/tasklet`](../examples/tasklet). Every problem found was fixed
in sixloom itself (CLI, skills or templates) and re-verified on a fresh
project.

## What was run

| Step | Result |
|---|---|
| `sixloom init` (react-vite, fumadocs, claude design) | clean repo with one `sxl(init)` commit |
| spec → design → preview | MSW preview with every state reachable by `?sxl=`, smoke-tested with Playwright |
| data → system → api | contract linted (Redocly), mocks validated against it, Fumadocs site built; the docs search finds `F-001.AC1` on both the data model and the `listTasks` endpoint |
| tests | 5 e2e tests green on the preview, mutation-checked (a broken behavior makes exactly the right tests fail); 4 contract tests red with 501; 9 tests locked |
| build | Hono + `node:sqlite`, TDD; full `just ci` green in real mode |
| validate | preview-versus-real screenshots of 4 states within 0.5%, no serious axe violations, docs build ok, `sixloom done` all green, then signed off |
| patch | CH-0001 went feedback → accepted → spec → screens, preview and locked test (re-locked) → built, and the looms were re-approved |
| matrix | `init` for all 45 front end × docs × design combinations with 7 agents and `--bank`: every catalog skill installed exactly once (native or bank), clean tree |
| adopt | `init --adopt` in a dirty repo commits only sixloom's files, leaving the user's edits, staged files and own skills untouched |
| agents | Codex, OpenCode, Gemini CLI, Kimi and Claude Code discover all installed skills (`scripts/smoke-harnesses.sh`) |
| TUI | the dashboard runs in a real PTY: renders, switches tabs, quits cleanly |

## Bugs found and fixed

| # | Found while | Problem | Fix |
|---|---|---|---|
| 1 | init inside a repo | created a nested `.git` | detect `git rev-parse --is-inside-work-tree` |
| 2 | init | scaffold never committed; context packs not ignored | `sxl(init)` commit of sixloom's paths; `.sixloom/context/` ignored |
| 3 | `… \| head` | panic on broken pipe | restore default SIGPIPE |
| 4 | spec gate | the unfilled template passed | the spec check rejects `<Placeholder>` and `Given …` |
| 5 | change list | `affects` printed as YAML | sequences joined with `, ` |
| 6 | preview | MSW 3 renamed `onUnhandledRequest`; Babel-based `data-sxl` stripping does not apply on Vite 8 / plugin-react 6 | skill updated; framework-agnostic `sxl-strip` Vite plugin (verified: 0 `data-sxl` in the production bundle) |
| 7 | api | `openapi-typescript` peer conflict with TypeScript 7 | run it through `npx`; documented |
| 8 | api | the contract gate silently checked nothing (MSW 3 internals changed) | gate rewritten on public `msw/node` and fails on undocumented statuses; shipped as a template |
| 9 | api | the gate caught non-UUID fixture IDs | skill requires realistic fixture formats |
| 10 | api | the Redocly default rules fail without `security`/`license` | documented in the skill |
| 11 | api | every docs scaffolder refuses a non-empty `docs/` | move-aside step in all three docs skills |
| 12 | update | the binary embedded stale skills (`include_dir!` doesn't track files) | `build.rs` with `rerun-if-changed` |
| 13 | loom end | files changed outside a loom's outputs were silently left uncommitted; the approval was written after the commit | `--all`, a left-over file warning, approval recorded before the commit |
| 14 | tests | the real-mode run reused a leftover mock server on the same port | separate ports per mode in the template |
| 15 | tests | skeleton assertions were flaky against a fast local API | the real-mode fixture adds the mock latency (setup only, so the lock is intact) |
| 16 | patch | one AC edit made every loom stale with no detail | per-feature spec hashing: `changed: spec/SPEC.md#F-001` |
| 17 | patch | re-approving 8 looms made 8 commits | `sixloom approve data system api` makes one commit and is atomic on typos |
| 18 | adopt | the init commit swept in the user's own untracked skill | stage only the skill dirs sixloom installed (regression test added) |
| 19 | smoke test | OpenCode seemed to miss skills | OpenCode truncates piped output; the smoke test reads from a file |

## Not covered by this run

- Real model sessions in each agent. The smoke test checks discovery offline,
  without API keys; the looms were executed by following the skills step by
  step.
- Parallel subagents and a PR per slice in `sxl-build`. The example has two
  slices, merged locally with no remote.
- Docusaurus and Starlight were verified by a separate scaffold-and-build
  run, not inside the Tasklet project.
- Stitch, which needs `stitch login` with a Google account.
