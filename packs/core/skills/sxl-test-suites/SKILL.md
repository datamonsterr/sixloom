---
name: sxl-test-suites
description: The sixloom test-suite loom, the first step of the build. It generates the acceptance suites before implementation (one Playwright e2e test per acceptance criterion, proven green against the preview, plus contract and integration tests per API operation, red until implemented), the backend skeleton and unit-test harness, the justfile, and fast, cached, parallel GitHub Actions CI. It then locks the acceptance tests. Use when the preview and API looms are approved, or when the user says "/sxl-test-suites", "generate tests", or "set up CI".
---

# sxl-test-suites: the test-suite loom

Follow the **sxl-loom** protocol. Loom ID: `tests`. It starts once the
`api` loom is approved (the preview was approved before it). Run it in a new
session. The context pack is `spec/SPEC.md`, `design/SCREENS.md`,
`contracts/openapi.yaml`, `apps/web/PREVIEW.md` and `system-design.mdx`
(ADRs only).

Load `playwright-cli` (or `webapp-testing`) for e2e, and `sxl-tdd` for the
test-quality rules.

## Tiers

| Tier | Location | One test per | Must be, at loom end |
|---|---|---|---|
| E2E | `tests/e2e/<feature>.spec.ts` | acceptance criterion | **green** against the preview (mock mode) |
| Contract | `tests/contract/<operationId>.spec.ts` | operation × documented status | **red** (API returns 501) |
| Integration | `tests/integration/` | key flow across endpoints | **red** |
| Unit | inside `apps/*` | written during the build | harness only, plus 1 example |

### E2E rules

- The test title starts with the AC ID: `test('F-003.AC2 shows an error when the name is empty', …)`.
  `sixloom trace` reads these titles.
- Select elements by `data-testid="<screen>.<control>"`, role, or label.
  Never use CSS structure or text that the copy may change.
- Each test sets its own state: in mock mode with `?sxl=<scenario>` and the
  mock db, in real mode through API calls in a fixture. The fixture switches
  on `E2E_MODE=mock|real`, so **the same file runs in both modes**.
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

Scaffold `apps/api` with the stack from the ADRs. Generate a route for
every operation that returns `501 { code: "not_implemented" }`, the database
migration tooling with an empty first migration, the seed script, and one
passing example unit test. Now `just ci-fast` runs end to end, and every
contract test fails for the right reason.

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
sixloom tests lock     # hashes tests/e2e, tests/contract and tests/integration into .sixloom/tests.lock
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
