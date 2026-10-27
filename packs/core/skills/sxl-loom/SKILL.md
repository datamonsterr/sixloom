---
name: sxl-loom
description: The sixloom loom protocol, shared by every sxl-* skill. It says how to start a loom, iterate with the human, pass the gate, save and commit, and detect stale looms. Use it when starting or ending any sixloom phase, when the user asks "where are we" or "what next", or when another sxl-* skill says to follow the loom protocol.
---

# The loom protocol

A **loom** is one phase of the sixloom framework. It produces one reviewed
artifact. The looms are:

| Loom | Skill | Artifact | Reviewer |
|---|---|---|---|
| `spec` | sxl-specify | `spec/SPEC.md` | product |
| `design` | sxl-design | `design/DESIGN.md`, `design/SCREENS.md`, `design/exports/` (`.dc.html` or Stitch HTML) | product |
| `preview` | sxl-preview | every interface (web, TUI, CLI, …) running on mocks | product |
| `data` | sxl-data-model | `docs/<content>/architecture/data-model.mdx` | engineer |
| `system` | sxl-system-design | `docs/<content>/architecture/system-design.mdx` | engineer |
| `api` | sxl-api-docs | `contracts/openapi.yaml` and the docs site | engineer |
| `tests` | sxl-test-suites | `tests/`, `.github/workflows/`, `.sixloom/tests.lock` | agent and engineer |
| `build` | sxl-build | working product | agent |
| `validate` | sxl-validate | `.sixloom/runs/validate-*.md` | product |

Order: the **product track** finishes first (`spec → design → preview`), so the
front end is final before any backend work starts. The **engineering track**
then derives the backend from the approved preview (`data → system → api`).
After that come `tests → build → validate`. A fix to the preview always
updates the spec in the same loom. Feedback goes through **sxl-patch**, never
straight into code.

The user drives all of this from the chat through the **sixloom** skill. You
run every `sixloom` command yourself; the user never needs the terminal.

## 1. Start

1. Run `sixloom loom start <loom>`. It refuses if an upstream loom is not
   approved, marks this loom `in_progress`, and writes the **context pack**
   `.sixloom/context/<loom>.xml`, which holds every input file in one
   document, with token counts, in the repomix layout. Read that one file
   and nothing else from the repo.
   - No CLI? Read `.sixloom/state.json` and check that the upstream looms
     have `"status": "approved"`. Then read only the upstream artifacts
     listed in the table above.
2. Read `sixloom.toml` for the project's choices (front-end framework, docs
   engine, design tool, harnesses).
3. If `sixloom status` lists this loom as `stale`, update only what the
   changed input affects. Do not regenerate the whole artifact.

## 2. Iterate

- Write the artifact first, then ask. A draft the human can react to beats a
  questionnaire.
- Keep each round small. Show what changed, then ask at most five numbered
  questions, each with your recommended answer, worded so that "yes" accepts
  it. Use the `grilling` skill format when the scope is unclear.
- Every artifact cites spec IDs (`F-001`, `F-001.AC2`) **without changing
  product behavior**, using one marker syntax (see "Trace markers" below).
  Never invent behavior that has no ID. If the work reveals a missing
  behavior, propose a spec change through **sxl-patch** and wait.
- Look up facts yourself (files, tool versions, schemas). Ask the human only
  for decisions.

## 3. Gate

Show the loom's **gate checklist** (each skill defines one). It is at most
five yes/no questions for its reviewer, written so that a non-engineer can
answer the product ones. Checks that a tool can do (ID coverage, schema
validity, mock conformance, broken links) are not questions for the human.
Run those checks yourself before showing the gate.

When the human says yes to every item, run:

```bash
sixloom loom end <loom>      # validates the artifacts, records input hashes
sixloom approve <loom>       # records the human approval (git author)
```

`loom end` commits the loom's declared outputs as `sxl(<loom>): <summary>`, and
lists any other changed files it left out. If the loom changed them on
purpose (for example, the api loom touches an interface's client types), end
it with `--all`. Use `--no-commit` to only mark it in review.

After a change, re-approve the looms you checked and found unaffected in one
go: `sixloom approve data system api` makes a single commit. No CLI? Update `.sixloom/state.json` (set the
loom's `status` to `approved` and its `approved_at` timestamp) and commit with
the same message format.

## 4. Hand off

End the session with one line naming the next loom and the command to start
it, for example: "Next: run `/sxl-preview` in a **new session**." Each loom
runs in a fresh session to keep context small. The context pack is what makes
that cheap.

## Trace markers

IDs never appear in API paths, payloads, or application logic. They live
only here:

| Where | Syntax | How a human finds it |
|---|---|---|
| Docs (`SCREENS.md`, `data-model.mdx`, `system-design.mdx`, …) | `<sub>sxl: F-003.AC1 · F-003.AC2</sub>` under the heading it belongs to | visible small print, so the docs site search finds it |
| OpenAPI operation `description` | last line `sxl: F-003.AC1, F-003.AC2` | the API reference page, so docs search finds it |
| Front end | `data-sxl="F-003.AC1"` on the element that delivers the AC | browser devtools (inspect element); stripped from production builds |
| Tests | the title starts with the AC ID: `test('F-003.AC1 …')` | test reports |

`sixloom trace F-003` joins all of these into one row per AC and lists the
gaps.

## Stale looms

`sixloom status` compares the input hashes recorded at approval with the
current files. When an input changes, every downstream loom becomes `stale`.
A stale loom is not wrong; it needs a targeted update and a new gate.

## Rules

- **Paths come from `sixloom layout`, never from memory.** Interfaces (web,
  TUI, CLI, mobile, desktop) and services each have a folder recorded in
  `sixloom.toml`. Run `sixloom layout --json` before writing code, and change
  the layout only with `sixloom layout interface|service|pattern …`, which also
  refreshes the rules block and the skills. The fixed folders are `spec/`,
  `design/`, `contracts/`, `docs/`, `tests/` and `.sixloom/`.

- `spec/SPEC.md` is the single truth for behavior. Every other artifact
  follows it, and where they disagree, the spec wins until a change record
  says otherwise.
- `contracts/openapi.yaml` is the single truth for data shape (once the `api`
  loom is approved).
- Never edit an approved artifact silently. Edits go through a change record
  (sxl-patch).
- Commit only the loom's artifacts and `.sixloom/`. Never commit secrets.
