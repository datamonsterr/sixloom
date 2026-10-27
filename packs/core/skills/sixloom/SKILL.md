---
name: sixloom
description: The entry point for sixloom. The user drives the whole framework from the chat, and you run the `sixloom` CLI for them. Use whenever the user says "sixloom …" or "/sixloom …" (init, status, next, spec, design, preview, data model, system design, api, tests, build, patch, validate, done, approve), asks where the project stands, or asks to move to the next step. The user never has to open a terminal.
---

# sixloom: run the framework from the chat

The user talks; you operate. Map what they say to a CLI call and a loom
skill, run the command yourself in the shell, and report the result in one
or two lines. Never tell the user to run a command; run it. The only things
you ask the user for are **decisions** and **gate answers**.

## 0. Make sure the CLI exists

Run `sixloom --version`. If it is missing, install it, then continue:

```bash
curl -fsSL https://raw.githubusercontent.com/datamonsterr/sixloom/main/install.sh | sh
# or, with Rust installed:  cargo install --git https://github.com/datamonsterr/sixloom sixloom
```

The agent's shell is not interactive, so always pass flags (`--yes`) to
commands that would otherwise open the terminal UI.

## 1. Route the request

| User says | You run | Then |
|---|---|---|
| `sixloom init` / "set up sixloom" | Ask the init questions in one round (below), then `sixloom init --yes --agents … --interface web:react-vite --docs … --design …` (add `--adopt` if the repo already has code) | Summarize what was created, then route to `spec` |
| `sixloom status` / "where are we" | `sixloom status` | Show the three track lines and the next step |
| `sixloom next` / "continue" / "what now" | `sixloom next` | Route to the loom it names |
| `sixloom spec` / "write the spec" | `sixloom loom start spec` | Follow **sxl-specify** |
| `sixloom design` / "UI design" | `sixloom loom start design` | Follow **sxl-design** |
| `sixloom preview` / "front end" | `sixloom loom start preview` | Follow **sxl-preview** |
| `sixloom data` / "data model" | `sixloom loom start data` | Follow **sxl-data-model** |
| `sixloom system` / "system design" | `sixloom loom start system` | Follow **sxl-system-design** |
| `sixloom api` / "API docs" / "docs site" | `sixloom loom start api` | Follow **sxl-api-docs** |
| `sixloom tests` | `sixloom loom start tests` | Follow **sxl-test-suites** |
| `sixloom build` / "implement it" | `sixloom loom start build` | Follow **sxl-build** (autonomous) |
| `sixloom validate` / "are we done" | `sixloom loom start validate`, then `sixloom done` | Follow **sxl-validate** |
| feedback, a bug, "change X" | `sixloom change new "<title>" --kind feedback\|bug --affects F-…` | Follow **sxl-patch** |
| "approved" / "yes to all" at a gate | `sixloom loom end <loom> --approve -m "<summary>"` | Then `sixloom next` and offer it |
| "accept CH-0003" | `sixloom approve change CH-0003` | Apply it with **sxl-patch** |
| "trace F-003" | `sixloom trace F-003` | Explain the row and any gaps |
| `sixloom done` | `sixloom done` | Show the checklist; on an explicit "yes, done": `sixloom done --confirm` |
| "add my skill at ./x" | `sixloom skills add ./x` | |
| "where does the code go" / "show the layout" | `sixloom layout` | One line per interface and service folder |
| "add a TUI" / "add a mobile app" | `sixloom layout interface add tui:ratatui[:path]` | The rules block and the skills update |
| "move the API to crates/api" | `sixloom layout service remove api`, then `sixloom layout service add api --kind http-api --stack … --path crates/api` | Move the folder with `git mv` |
| "switch docs to starlight" (or any choice) | edit `sixloom.toml`, then `sixloom update` | |

If `loom start` refuses because an upstream loom is not approved, say which
one, and offer to run it first. Pass `--force` only when the user insists.

### Init questions (one round, recommended answers marked)

1. New project, or add sixloom to the existing code here? (Detect this: if
   there is code, recommend adopt.)
2. Which coding agents will the team use? (Recommend the one you are running
   in, plus `codex`.)
3. Interfaces: what users touch, one or more of web (`react-vite`
   recommended, `nextjs`, `vue`, `nuxt`, `sveltekit`), TUI (`ratatui`,
   `bubbletea`, `textual`, `ink`), CLI (`clap`, `cobra`, `typer`,
   `commander`), mobile (`expo`, `flutter`) or desktop (`tauri`, `electron`).
   Pass each as `--interface kind:framework[:path]`. React web also takes a
   UI kit: `shadcn` or `none`. Backend services are not asked here; the
   system-design loom decides and records them.
4. Docs engine: `fumadocs` (recommended), `docusaurus` or `starlight`.
5. Design tool: `claude` (recommended), `stitch` (needs `stitch login`) or
   `none`.

## 2. Keep each loom in its own session

At the end of every loom gate, say in one line which loom is next. Mention
that a **new session** keeps context small, because `sixloom loom start`
writes a context pack (`.sixloom/context/<loom>.xml`) so the next session
starts from that one file. If the user prefers to keep going in this
session, continue: run `sixloom loom start <next>` and read only the pack.

## 3. Report briefly

After each command, give one line of outcome (for example, "spec approved;
design is next") and the next question for the user, if any. Do not paste
whole CLI outputs unless asked. The user can always open the dashboard
themselves with `sixloom` in a terminal, but they never have to.
