---
name: sxl-test-suites
description: The sixloom test-suite loom, the first step of the build. It generates the acceptance suites before implementation (one e2e test per acceptance criterion and interface, in the interface's own test tool, proven green against the preview, plus contract and integration tests per API operation, red until implemented), the backend skeleton and unit-test harness, the justfile, and fast, cached, parallel GitHub Actions CI. It then locks the acceptance tests. Use when the preview and API looms are approved, or when the user says "/sxl-test-suites", "generate tests", or "set up CI".
---

# sxl-test-suites: the test-suite loom

Follow the **sxl-loom** protocol. Loom ID: `tests`. It starts once the
`api` loom is approved (the preview was approved before it). Run it in a new
session. The context pack is `spec/SPEC.md`, `design/SCREENS.md`,
`contracts/openapi.yaml`, every interface's `PREVIEW.md` and
`system-design.mdx` (ADRs only). Read the folders from `sixloom layout --json`.

Load `sxl-tdd` for the test-quality rules. Pick the e2e tool by interface kind:

| Interface kind | E2E tool | AC in the test name |
|---|---|---|
| web, desktop (Electron) | Playwright (`playwright-cli` skill); start from the verified templates in [files/](files/) | `test('F-001.AC1 shows …')` |
| TUI | the framework harness: Ratatui `TestBackend`, Textual `Pilot`, Bubble Tea `teatest`, Ink `ink-testing-library`; or the real binary in a pseudo-terminal | `fn f001_ac1_shows_tasks()` / `def test_f001_ac1_…` / `func TestF001AC1…` |
| CLI | run the built binary with the mock or real API; assert stdout and the exit code | same naming |
| mobile | Maestro or Detox against the Expo/Flutter build | flow name starts with the AC ID |

`sixloom tests lock` and `sixloom trace` read JS/TS, Rust, Python and Go
tests. They map names such as `f001_ac1_…` back to `F-001.AC1`, and they treat
`.skip`, `#[ignore]`, `@pytest.mark.skip` and `t.Skip` as disabling a test.
If a language needs its tests inside its own package (a Rust TUI's
`crates/tui/tests/acceptance`), add that folder to `[tests] locked` in
`sixloom.toml`.

For web, pin `@playwright/test` to the version of the browsers you have
(`npx playwright --version`).

## Tiers

| Tier | Location | One test per | Must be, at loom end |
|---|---|---|---|
| E2E | `tests/e2e/<interface-id>/…` | acceptance criterion × interface | **green** against the preview (mock mode) |
| Contract | `tests/contract/<operationId>.spec.ts` | operation × documented status | **red** (API returns 501) |
| Integration | `tests/integration/` | key flow across endpoints | **red** |
| Unit | inside each interface and service folder | written during the build | harness only, plus 1 example |

### E2E rules

- The test title starts with the AC ID: `test('F-003.AC2 shows an error when the name is empty', …)`.
  `sixloom trace` reads these titles.
- Select elements by `data-testid="<screen>.<control>"`, role, or label.
  Never use CSS structure or text that the copy may change.
- Each test sets its own state through one fixture, `open(scenario)`. In
  mock mode it is `?sxl=<scenario>`. In real mode it resets state through a
  **test-only seam** (`POST /api/__test/reset`, registered only when
  `NODE_ENV=test`, and never in the contract), seeds through the public API,
  and forces server errors with `page.route`. The fixture switches on
  `E2E_MODE=mock|real`, so **the same file runs in both modes**.
- **Transient states** (skeletons, "submitting") need latency. The real-mode
  fixture delays the data requests like the mocks do, or a fast local API
  makes those assertions flaky. That is a setup fix, so it never touches
  assertions.
- **Different ports per mode** (mock 5173, real 5174). Otherwise
  `reuseExistingServer` silently runs the "real" suite against a leftover
  mock server.
- No `waitForTimeout`. Wait for a locator or a response.
- **Prove it**: run the suite against the preview (`just e2e-mock`). Every
  test must pass. A failing test means the test is wrong or the preview is
  missing behavior. In the second case, fix the preview or raise a change
  record. Then break the behavior on purpose for 2–3 tests and confirm that
  they fail, as a check that they assert something.

### Contract tests

For each operation and each documented status, send the request and assert
the status code and the body against the contract schema (ajv). Use the
`examples` from `openapi.yaml` as inputs. The contract is the oracle; never
copy the implementation's output.

### Backend skeleton ("dummy code")

Scaffold every service from `sixloom layout` in its recorded folder, with the
stack from the ADRs. Generate a route for
every operation that returns `501 { code: "not_implemented" }`, a `/api/health`
route (the Playwright `webServer` waits on it), the test-only reset seam, the
database migration tooling with an empty first migration, the seed script, and
one passing example unit test. Now `just ci-fast` runs end to end, and every
contract test fails for the right reason. Check it:
`npx playwright test --project=contract | grep Received` shows only 501.

End this loom with `sixloom loom end tests --all`.

## Task runner and CI

Copy [justfile.template](justfile.template) to `justfile` and fill in the
commands for the chosen stack. Copy [ci.yml.template](ci.yml.template) to
`.github/workflows/ci.yml`. CI calls **only `just` targets**, so local and
CI behavior are identical. The template already has these properties:

- **Fast path first**: a `changes` job filters by path. Web and API jobs
  run only when their files change. Lint, typecheck and unit run in parallel
  jobs, not sequentially.
- **Async and sharded**: e2e runs as a 4-shard matrix against the real stack
  (service container database), with Playwright traces uploaded on failure.
- **Caching**: package manager caches, the Playwright browser cache, and the
  build cache keyed by lockfile.
- **Cancel superseded runs** with `concurrency`.
- **Lock check**: the `sixloom tests verify` job fails if a locked test was
  weakened.
- Failures print the exact `just` command that reproduces them locally.

## Lock

When every e2e test is green on the preview and every contract test is red
for the right reason, run:

```bash
sixloom tests lock     # fingerprints every folder in [tests] locked into .sixloom/tests.lock
```

Locking hashes each test's normalized assertions. The build may then repair
selectors and waits, but not change what a test asserts (see `sxl-build`).

## Gate (engineer reviewer)

1. Does `sixloom trace` show every `must` AC with at least one e2e test?
2. Do the e2e tests read like the spec (would you accept the product if they
   pass)?
3. Is every contract test red with `not_implemented` and not for another
   reason?
4. Does CI on this branch finish in under 10 minutes, with all jobs either
   green or expected red?
5. Do you accept the locked set?

## Next

`/sxl-build` in a new session. It runs autonomously.
