# Build plan

| Slice | Depends on | Tests | Status |
|---|---|---|---|
| F-001 See my tasks | — | listTasks contract, F-001.AC1–3 e2e | done |
| F-002 Add a task | F-001 (Task table) | createTask contract ×3, F-002.AC1–2 e2e | done |

## Run summary

- Slices: 2 done, 0 blocked. Sequential (no subagents needed for 2 slices); merged locally (`build.pr = true` but no remote in dogfood).
- Tests added: 2 unit (store ordering, validation). Locked tests unchanged (`sixloom tests verify` ✓).
- Fixture fix: real-mode latency route so skeleton states are observable (setup only, fingerprints unchanged).
- Change records raised: none.
