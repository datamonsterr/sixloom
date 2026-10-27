<!-- sixloom:begin (managed by `sixloom`; edit outside this block) -->
## sixloom

This project is built with the [sixloom](https://github.com/datamonsterr/sixloom) loom workflow.
Run `sixloom status` to see where it stands and `sixloom next` for what to do next.

- **Looms**, one per session: spec → design → preview (product track); spec → data → system → api (engineering track); then tests → build → validate. Each has an `sxl-*` skill; follow `sxl-loom` to start and end one.
- **Truth**: `spec/SPEC.md` for behavior (IDs `F-001`, `F-001.AC1`), and `contracts/openapi.yaml` for data shape. Cite IDs everywhere.
- **Changes** go through `sxl-patch` (`spec/changes/CH-xxxx`), never straight into code.
- **Locked tests** (`.sixloom/tests.lock`): never delete, skip or weaken them; `sixloom tests verify` runs in CI.
- **Before every push**: `just ci-fast`.

Layout: `spec/` spec and changes · `design/` design system, screens, comps · `apps/web` preview → real front end · `apps/api` backend · `contracts/` OpenAPI · `docs/` docs site (docs/content/docs) · `tests/` e2e, contract, integration · `.sixloom/` state, locks, run logs.

Choices: front end **react-vite** (none), docs **fumadocs**, design **claude**. Your own skills can sit next to the sixloom ones in `.claude/skills/`, `.agents/skills/`.
<!-- sixloom:end -->
