# sixloom

**sixloom turns an agreed solution into a working product with coding agents.**
Humans shape the goal while it is still cheap to change: a short feature list,
a clickable front end, a database diagram and an API reference. Agents then
build the rest test-first, until the real app behaves exactly like the preview
and the docs.

It ships as skills (for Claude Code, Codex, OpenCode, Gemini CLI, Kimi and
other agents) plus one small Rust CLI. You drive it by talking to your agent:

```
you    › /sixloom init
agent  › (asks 5 setup questions, runs `sixloom init`, installs the skills)
you    › sixloom spec
agent  › (drafts spec/SPEC.md from your user stories and asks what's unclear)
you    › approved
agent  › spec approved and committed. Next: UI design. Start a new session and say "sixloom design".
```

You never have to open the terminal. The agent runs `sixloom` for you.

<!-- framework diagram goes here -->

---

## Why

A "good" product from agents means three things at once:

1. It **looks and works as expected**.
2. **Development is fast** from start to finish.
3. **Few tokens** are spent.

Most agent projects fail on the first point. The agent built something, just
not what you meant, and every round of "no, not like that" costs time and
tokens. sixloom moves all human judgment to the front, to the artifacts a
person can review in minutes, and makes everything after that mechanical.

### Mindset

- **Small, fast feedback loops.** Every step is draft → react → revise, in one
  short session.
- **Review only what is easy for humans to review.** A few features, not 100
  files. A front end you click. A diagram. An API reference you search. Never
  the code.
- **Put human attention on the goal**, and review the result before it
  exists: the preview and the docs *are* the result, seen early.

It assumes problem discovery is already done: you arrive with user stories or
use cases and an initial idea of the solution.

## How it works

The work is split into **looms**. A loom is one agent session that produces
one artifact, loops with you until it is right, ends at a short yes/no gate,
and is committed.

### Phase 1: you and the agent shape the goal

**Product track.** This runs first and finishes first.

| Loom | You say | Produces | You review |
|---|---|---|---|
| Spec | `sixloom spec` | `spec/SPEC.md`: features with IDs and acceptance criteria | a short feature list |
| UI design | `sixloom design` | `design/SCREENS.md` (every screen, button, state, flow), `design/DESIGN.md` (design system), comps from Claude Design (`.dc.html`) or Stitch | flows and key screens |
| Preview | `sixloom preview` | the **real front end** in your framework, on mock data, with loading skeletons, empty states and errors | you click it |

The preview is where the product gets finished. Ask for changes until it is
right. Every change that alters behavior updates the spec in the same round.

**Engineering track.** This starts from the approved preview.

| Loom | You say | Produces | You review |
|---|---|---|---|
| Data model | `sixloom data` | entities, relationships and rules, with a rendered ER diagram | a diagram |
| System design | `sixloom system` | services, flows, stack decisions (Mermaid, plus a draw.io file) | a diagram |
| API & docs | `sixloom api` | `contracts/openapi.yaml` and a docs site (API reference, architecture, setup, codebase tour) | the docs site |

### Phase 2: agents build it

| Loom | You say | What happens |
|---|---|---|
| Test suites | `sixloom tests` | One Playwright test per acceptance criterion, **proven green on the preview**. Contract tests per endpoint (red until built). A backend skeleton, a `justfile`, and fast GitHub Actions CI. The acceptance tests are **locked**. |
| Build | `sixloom build` | Autonomous TDD in vertical slices: parallel subagents in separate worktrees, a PR per slice, merge when CI is green. It stops when every locked test passes against the real backend. |
| Validate | `sixloom validate` | Checks the real app against the preview and the docs (tests, screenshots, accessibility, an exploratory pass) and fixes gaps. **You** decide when it is done. |

### After that: feedback → patch → changelog stack

Say "change X" or report a bug. The agent writes a change record
(`spec/changes/CH-0007-*.md`) and updates the spec first, then the affected
looms, then the locked tests, and only then the code. Stale looms are detected
automatically.

The full reasoning, with each weak point found in review and the decision
taken, is in [docs/FRAMEWORK.md](docs/FRAMEWORK.md).

## Install

Pick **one** of the three ways. All three give your agent the same skills.

### 1. The CLI (any agent; recommended)

```sh
curl -fsSL https://raw.githubusercontent.com/datamonsterr/sixloom/main/install.sh | sh
# or: cargo install --git https://github.com/datamonsterr/sixloom sixloom
```

The single binary embeds every skill and template, so it needs no network
after install. Then, in your project, either tell your agent "sixloom init"
(once the `sixloom` skill is installed) or run it yourself:

```sh
sixloom init                 # interactive wizard in a terminal
sixloom init --yes --agents claude,codex --frontend react-vite --docs fumadocs --design claude
sixloom init --adopt         # add sixloom to an existing codebase; app code is untouched
```

`init` installs **only** the skills for your choices (front end, docs engine,
design tool) into each agent's skills directory, and writes a short managed
block into `CLAUDE.md` / `AGENTS.md` / `GEMINI.md`.

### 2. Claude Code plugin

```sh
claude plugin marketplace add datamonsterr/sixloom
claude plugin install sixloom@sixloom            # the looms + TDD + grilling + Playwright
claude plugin install sixloom-react@sixloom      # pick your variants:
#   sixloom-react | sixloom-nextjs | sixloom-vue | sixloom-nuxt | sixloom-svelte | sixloom-shadcn
#   sixloom-fumadocs | sixloom-docusaurus | sixloom-starlight | sixloom-stitch
```

Then say `/sixloom init` in a session. The skill installs the CLI if it is
missing.

### 3. Codex plugin

```sh
codex plugin marketplace add datamonsterr/sixloom
codex plugin add sixloom@sixloom
codex plugin add sixloom-frontend@sixloom        # also: -engineering -design -docs -testing
```

## Supported agents

`sixloom init --agents …` writes skills where each agent looks for them.
Skills follow the open [Agent Skills](https://agentskills.io) format
(`<name>/SKILL.md`).

| Agent | `--agents` | Skills dir | Instructions | Verified |
|---|---|---|---|---|
| Claude Code | `claude` | `.claude/skills` | `CLAUDE.md` | ✅ plugin validate + install |
| OpenAI Codex CLI | `codex` | `.agents/skills` | `AGENTS.md` | ✅ skills in prompt input + plugin install |
| OpenCode | `opencode` | `.agents/skills` | `AGENTS.md` | ✅ `opencode debug skill` |
| Gemini CLI | `gemini` | `.agents/skills` | `GEMINI.md` | ✅ `gemini skills list` |
| Kimi Code CLI | `kimi` | `.agents/skills` | `AGENTS.md` | ✅ kimi's own skill discovery |
| Google Antigravity | `antigravity` | `.agents/skills` | `AGENTS.md` | from docs |
| Cursor | `cursor` | `.cursor/skills` | `AGENTS.md` | from docs |
| GitHub Copilot CLI | `copilot` | `.github/skills` | `AGENTS.md` | from docs |
| Amp | `amp` | `.agents/skills` | `AGENTS.md` | from docs |

"Verified" means [`scripts/smoke-harnesses.sh`](scripts/smoke-harnesses.sh)
installs the real CLI and checks, offline and with no API key, that it
discovers every skill `sixloom init` wrote. CI runs this on every change.
Details are in [docs/HARNESSES.md](docs/HARNESSES.md).

## Using it day to day

Talk to your agent. The `sixloom` skill maps what you say to the CLI and the
right loom skill:

| Say | Happens |
|---|---|
| `sixloom status` / "where are we?" | the three tracks with status: approved, in progress, stale, ready |
| `sixloom next` / "continue" | starts the next loom |
| `sixloom spec` … `sixloom validate` | starts that loom (it refuses if an earlier one is not approved yet) |
| "approved" / "yes to all" at a gate | `sixloom loom end <loom> --approve`, which commits `sxl(<loom>): …` |
| "change the export to CSV" / "bug: …" | a change record, then the patch flow |
| `sixloom trace F-003` | where feature F-003 lives: screens, UI elements, endpoints, docs, tests |
| `sixloom done` | the done checklist; it records your sign-off only when you say so |

**Use a new session per loom.** `sixloom loom start` writes a context pack,
`.sixloom/context/<loom>.xml`: every input file the loom needs and nothing
else, with token counts, in the [repomix](https://github.com/yamadashy/repomix)
layout. A fresh session reads one file instead of exploring the repo.

Prefer a dashboard? Run `sixloom` in a terminal:

- **Looms tab**: the pipeline, the status of each loom, why a loom is stale,
  its context pack, and the exact invocation for each of your agents. `s`
  starts a loom, `e` ends it, `a` approves it.
- **Trace tab**: the feature → screen → UI → endpoint → docs → test matrix,
  with gaps.
- **Changes tab**: the changelog stack.
- **Skills tab**: what is installed and what is available.

### Tracing without touching the product

Every feature has an ID (`F-003`), and every acceptance criterion has one
(`F-003.AC1`). IDs never appear in API paths, payloads or app logic, only as
markers:

| Where | Marker | You find it with |
|---|---|---|
| docs (screens, data model, system design) | `<sub>sxl: F-003.AC1</sub>` | the docs site search |
| OpenAPI operation description | last line `sxl: F-003.AC1` | the API reference and its search |
| front end | `data-sxl="F-003.AC1"` (stripped in production builds) | browser devtools |
| tests | `test('F-003.AC1 …')` | test reports |

To trace a feature, inspect an element in the preview, copy its ID, and paste
it into the docs search. `sixloom trace` checks the whole matrix.

### Locked tests

The acceptance tests are written from the spec, proven on the preview, then
locked. During the build an agent **may** add tests and repair a selector or
a wait. It **may not** delete, rename, skip or weaken a locked test.
`sixloom tests verify` (run in CI) compares each test's assertion
fingerprint. A locked test that looks wrong becomes a change record that you
approve.

### When is it done?

`sixloom done` checks these:

- every loom is approved and none is stale;
- the locked tests are intact;
- the trace is complete;
- the changelog stack is empty;
- the validate report passed: real-mode e2e, preview-versus-real screenshots,
  docs commands, no console errors, no serious accessibility violations, CI
  green, and nothing blocked.

It then lists what only you can confirm. It is done when you say so.

## Project layout

```
sixloom.toml            your choices: agents, front end, docs engine, design tool, build limits
AGENTS.md / CLAUDE.md   managed sixloom block + your own rules
spec/
  SPEC.md               the truth for behavior (features, ACs, IDs)
  changes/CH-*.md       change records (the changelog stack)
  CHANGELOG.md          generated from the records
design/
  DESIGN.md             design system tokens and rules
  SCREENS.md            every screen × state × control, plus flows
  exports/              comps: *.dc.html (Claude Design) or Stitch HTML
apps/
  web/                  the preview, which becomes the real front end (src/api, src/mocks)
  api/                  the backend (scaffolded by the tests loom)
contracts/openapi.yaml  the truth for data shape
docs/                   docs site (Fumadocs | Docusaurus | Starlight), MDX pages
  diagrams/system.drawio
tests/
  e2e/                  one test per AC (locked)
  contract/ integration/  per operation and flow (locked)
.sixloom/
  state.json            loom status, approvals, input hashes (stale detection)
  context/<loom>.xml    context packs
  tests.lock            locked-test fingerprints
  runs/                 build plan and log, validate reports
justfile                the one command surface for humans, agents and CI
```

## Choices and skill packs

| Choice | Options | Default |
|---|---|---|
| Front end | `react-vite`, `nextjs`, `vue`, `nuxt`, `sveltekit` (UI kit: `shadcn` or `none`) | `react-vite` + `shadcn` |
| Docs engine | `fumadocs`, `docusaurus`, `starlight` | `fumadocs` |
| Design tool | `claude` (Claude Design), `stitch` (Google Stitch CLI), `none` | `claude` |

All three docs engines render MDX and Mermaid and generate the API reference
from `contracts/openapi.yaml`.

Skills come in **packs**: `core`, `engineering`, `design`, `frontend`,
`docs` and `testing`. Most are sixloom's own. The rest are well-known skills
vendored at a pinned revision with their licenses, from Matt Pocock (`tdd`,
ported as `sxl-tdd`; `grilling`; `diagnosing-bugs`), Anthropic
(`frontend-design`, `webapp-testing`), Vercel, antfu, Svelte, Microsoft
(`playwright-cli`) and Google (`stitch`). The full list is in
[docs/SKILLS.md](docs/SKILLS.md).

- **Change a choice later**: edit `sixloom.toml`, then run `sixloom update`.
  sixloom swaps only its own skills.
- **Keep the other variants handy without bloating context**:
  `sixloom init --bank` puts them in an
  [other-skill](https://github.com/datamonsterr/other-skill) bank
  (`.otherskill/bank/<pack>/…` + `otherskill.config.json`). other-skill's
  router then injects a skill only when a prompt needs it.
- **Bring your own skills**: they work alongside sixloom's. Drop them in your
  agent's skills directory, or run `sixloom skills add ./my-skill` to copy one
  into every configured agent. sixloom never touches skills it did not
  install.

## CLI reference

Everything works non-interactively, so agents can run it. The TUI opens only
in a real terminal.

```
sixloom                         dashboard (TUI)
sixloom init [DIR] [--adopt] [--agents a,b] [--frontend X] [--ui-kit X] [--docs X] [--design X] [--fidelity key|all|none] [--bank] [--yes]
sixloom status [--json]         every loom: approved / in review / in progress / ready / blocked / stale (+ changed inputs)
sixloom next                    next loom + the invocation for each of your agents
sixloom loom start <loom>       check prerequisites, write .sixloom/context/<loom>.xml, mark in progress
sixloom loom end <loom> [--approve] [-m msg] [--no-commit]   machine checks, then commit `sxl(<loom>): …`
sixloom loom check <loom>       machine checks only
sixloom approve <loom>          record human approval (git author) and input hashes
sixloom approve change CH-0003  accept a change record
sixloom change new "<title>" [--kind feedback|bug|sync|test] [--affects F-001.AC2]
sixloom change list | status <id> <status> | log
sixloom trace [F-003] [--json] [--strict]
sixloom tests lock | verify
sixloom skills list | sync | add <dir>
sixloom update                  re-install skills and instruction blocks after editing sixloom.toml
sixloom agents                  supported agents and whether each is installed
sixloom doctor                  tools, skills per agent, spec and lock health
sixloom done [--confirm]        done checklist; --confirm records your sign-off
```

Looms: `spec`, `design`, `preview`, `data`, `system`, `api`, `tests`,
`build`, `validate`.

## Developing sixloom

```
cli/                Rust CLI + TUI (clap, ratatui); embeds packs/ and templates/
packs/<pack>/skills/  every skill; packs/catalog.toml says which installs when
packs/<pack>/.codex-plugin/   Codex plugins (generated)
.claude-plugin/marketplace.json   Claude Code marketplace (generated)
.agents/plugins/marketplace.json  Codex marketplace (generated)
templates/project/  files `init` writes
scripts/vendor_skills.py   refresh vendored skills from their pinned upstreams (--check in CI)
scripts/gen_plugins.py     regenerate plugin manifests + docs/SKILLS.md (--check in CI)
scripts/smoke-harnesses.sh real-agent discovery smoke test
```

```sh
cargo test && cargo clippy --all-targets -- -D warnings
python3 -I scripts/gen_plugins.py         # after editing catalog.toml or a skill description
python3 -I scripts/vendor_skills.py       # after bumping an upstream rev
scripts/smoke-harnesses.sh target/debug/sixloom
```

To add a skill, put it in `packs/<pack>/skills/<name>/`, add an entry to
`packs/catalog.toml` (with `upstream = { repo, rev, path, license }` if it is
vendored), and regenerate.

## License

MIT for sixloom's own code and skills. Vendored skills keep their own
licenses (MIT or Apache-2.0), included next to each skill.
