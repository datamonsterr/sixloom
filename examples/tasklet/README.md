# Tasklet: a complete sixloom run

A two-feature todo app taken through every loom with the `sixloom` CLI. It
is the project used to dogfood sixloom v0.1: React + Vite front end (built
first as the MSW preview), a Hono + `node:sqlite` API, Fumadocs docs, and locked
Playwright suites. It also went through one feedback cycle (CH-0001).

Read it in this order, the same order the looms ran:

1. [`spec/SPEC.md`](spec/SPEC.md): 2 features, 5 acceptance criteria
2. [`design/SCREENS.md`](design/SCREENS.md), [`design/DESIGN.md`](design/DESIGN.md), [`design/exports/`](design/exports/)
3. [`apps/web`](apps/web), with [`PREVIEW.md`](apps/web/PREVIEW.md) for the state URLs
4. [`docs/content/docs/architecture/`](docs/content/docs/architecture/): data model and system design (`sxl:` markers)
5. [`contracts/openapi.yaml`](contracts/openapi.yaml) and [`scripts/check-mocks.mjs`](scripts/check-mocks.mjs)
6. [`tests/`](tests/): e2e (mock and real), contract, visual + axe, and [`.sixloom/tests.lock`](.sixloom/tests.lock)
7. [`apps/api`](apps/api), [`.sixloom/runs/`](.sixloom/runs/): build plan, validate report
8. [`spec/changes/`](spec/changes/): the feedback cycle

## Run it

```bash
sixloom update      # installs the skills (not checked in here)
just install
just preview        # the preview on mocks
just ci             # lint, typecheck, unit, contract gate, lock verify, contract + e2e (real)
just docs
```

## History (one commit per loom step)

| Commit (original run) | Step |
|---|---|
| `1030373` | sxl(init): add sixloom |
| `d8364cb` | sxl(spec): Tasklet v1 spec |
| `841f5a7` | sxl(design): Task list screen |
| `03325e0` | sxl(preview): Task list preview on MSW |
| `4a49342` | sxl(data): Task entity |
| `d11c735` | sxl(system): Web + Hono API + SQLite |
| `68ca541` | sxl(api): Tasklet API contract + Fumadocs site |
| `b844115` | sxl(api): reconcile preview with contract |
| `8228a07` | sxl(tests): locked e2e + contract suites, API skeleton, CI |
| `e5fcf41` | sxl(build): F-001 + F-002 implemented, locked suite green in real mode |
| `f616e28` | sxl(validate): validate report: pass |
| `412c8c8` | sxl(validate): approved by Claude <noreply@anthropic.com> |
| `b6cdb1d` | sxl(change): accept CH-0001 Empty state should say 'All clear' |
| `dead80d` | sxl(design): CH-0001: empty state says All clear |
| `ccfe8b4` | sxl(preview): approved by Claude <noreply@anthropic.com> |
| `22809f4` | sxl(data): approved by Claude <noreply@anthropic.com> |
| `32ce16e` | sxl(system): approved by Claude <noreply@anthropic.com> |
| `ea4a1c9` | sxl(api): approved by Claude <noreply@anthropic.com> |
| `93f8042` | sxl(tests): approved by Claude <noreply@anthropic.com> |
| `ea94fd4` | sxl(build): approved by Claude <noreply@anthropic.com> |
| `088a71b` | sxl(validate): approved by Claude <noreply@anthropic.com> |
| `a4a97e2` | sxl(design): approved by Claude <noreply@anthropic.com> |
| `960e1e0` | sxl(preview): approved by Claude <noreply@anthropic.com> |
| `b0af78f` | sxl(data): approved by Claude <noreply@anthropic.com> |
| `4e01236` | sxl(system): approved by Claude <noreply@anthropic.com> |
| `701fff2` | sxl(api): approved by Claude <noreply@anthropic.com> |
| `3052fb8` | sxl(tests): approved by Claude <noreply@anthropic.com> |
| `ae8dc34` | sxl(build): approved by Claude <noreply@anthropic.com> |
| `11481e2` | sxl(validate): approved by Claude <noreply@anthropic.com> |
| `a41fa8c` | sxl(data,system,api): approved by Claude <noreply@anthropic.com> |
