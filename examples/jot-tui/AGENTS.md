<!-- sixloom:begin (managed by `sixloom`; edit outside this block) -->
## sixloom

This project is built with the [sixloom](https://github.com/datamonsterr/sixloom) loom workflow.
Run `sixloom status` to see where it stands and `sixloom next` for what to do next.

- **Looms**, one per session: spec → design → preview (product track); data → system → api (engineering track, from the approved preview); then tests → build → validate. Each has an `sxl-*` skill; follow `sxl-loom` to start and end one.
- **Truth**: `spec/SPEC.md` for behavior (IDs `F-001`, `F-001.AC1`) and `contracts/openapi.yaml` for data shape. Cite IDs only as `sxl:` markers, never in product behavior.
- **Changes** go through `sxl-patch` (`spec/changes/CH-xxxx`), never straight into code.
- **Locked tests** (`.sixloom/tests.lock`): never delete, skip or weaken them; `sixloom tests verify` runs in CI.
- **Before every push**: `just ci-fast`.

### Layout (pattern: local-only)

Code lives only in the folders below; `sixloom.toml` is their source of truth and every sixloom command, check and test reads it. Never hard-code another path. To add, move or remove an interface or service, run `sixloom layout interface|service …` (it updates `sixloom.toml`, this block and the skills), then follow the new layout.

- `crates/tui/` interface **tui** (tui · ratatui): API client `crates/tui/src/client.rs`, mocks `crates/tui/src/mocks.rs`, preview guide `crates/tui/PREVIEW.md`
- no backend services: the interfaces own their data (local-only)
- locked acceptance tests: `tests/e2e/`, `tests/contract/`, `tests/integration/`, `crates/tui/tests/`

Fixed folders: `spec/` spec and changes · `design/` design system, screens, comps · `contracts/` API contract · `docs/` docs site (pages in `docs/src/content/docs`) · `tests/` locked e2e, contract, integration · `.sixloom/` state, locks, run logs.

Choices: docs **starlight**, design **none**. Your own skills can sit next to the sixloom ones in `.claude/skills/`, `.agents/skills/`.
<!-- sixloom:end -->
