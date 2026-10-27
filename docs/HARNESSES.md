# Agent harness support

`sixloom init --agents …` copies the selected skills into each agent's
project skills directory and writes a managed block into its instruction file.
When several agents share a directory (`.agents/skills`), the skills are
written once.

## How each path was established (October 2026)

| Agent | Version checked | Skills dir | Evidence |
|---|---|---|---|
| Claude Code | 2.x (`claude`) | `.claude/skills` | `claude plugin validate .` passes, and `claude plugin marketplace add` + `plugin install sixloom@sixloom` succeed. `.claude/skills` is Claude Code's documented project skills location. |
| Codex CLI | 0.164.0 | `.agents/skills` | `codex debug prompt-input` in an initialised project lists every sixloom skill. `codex plugin marketplace add` + `plugin add sixloom@sixloom` install the core pack. The binary also reads `.codex/skills`. |
| OpenCode | 1.x (`opencode-ai`) | `.agents/skills` | `opencode debug skill` lists every skill. OpenCode scans `.claude/skills` and `.agents/skills` up the tree, plus `.opencode/{skill,skills}`, and deduplicates by name. |
| Gemini CLI | `@google/gemini-cli` latest | `.agents/skills` | `gemini skills list` shows every skill as `[Enabled]` once the folder is trusted. Gemini reads both `.gemini/skills` and `.agents/skills`, and `.agents/skills` wins on conflicts, so sixloom writes only `.agents/skills`. |
| Kimi Code CLI | 1.52.0 | `.agents/skills` | kimi's own `resolve_skills_roots` + `discover_skills_from_roots` return every skill with scope `project`. It also reads the brand dirs `.kimi/skills`, `.claude/skills` and `.codex/skills`. |
| Antigravity | — | `.agents/skills` | Follows other-skill's setup, which targets the shared `.agents/skills` catalog for Antigravity. Not smoke-tested here (desktop app). |
| Cursor | — | `.cursor/skills` | Documented location; not smoke-tested. |
| Copilot CLI | — | `.github/skills` | Documented location; not smoke-tested. |
| Amp | — | `.agents/skills` | Documented location; not smoke-tested. |

Run [`scripts/smoke-harnesses.sh`](../scripts/smoke-harnesses.sh) to repeat
the checks. They need no API keys and make no model calls. CI runs the script
on every change.

## Invoking a skill

`sixloom next` prints the right form for each configured agent:

| Agent | Form |
|---|---|
| Claude Code | `/sxl-preview` (or just say "sixloom preview") |
| Codex | `$sxl-preview` |
| Kimi | `/skill:sxl-preview` |
| Others | "use the sxl-preview skill", or "sixloom preview" |

Every agent also picks skills on its own from their descriptions, so plain
requests like "sixloom preview" or "build the preview" work everywhere.

## Notes

- When `claude` and an `.agents/skills` agent are both selected, OpenCode and
  Kimi see each skill twice (once from `.claude/skills`). Both deduplicate by
  name, so this is harmless.
- `opencode debug skill | …` can truncate its JSON when stdout is a pipe, so
  some skills seem to be missing. Redirect it to a file first, as the smoke
  test does.
- Subagents (used by `sxl-build` for parallel slices) exist in Claude Code,
  Codex, OpenCode and Kimi. Elsewhere the build runs its slices one after
  another.
