---
name: sxl-tdd
description: Test-driven development for sixloom builds. It runs a red → green loop in vertical slices at seams already fixed by the API contract and the locked acceptance tests, and it follows the locked-test rules. Use when implementing a slice in sxl-build, fixing a bug from sxl-patch, or whenever the user wants test-first work, red-green-refactor, or integration tests.
---

# sxl-tdd

> Ported from Matt Pocock's [`tdd`](https://github.com/mattpocock/skills/tree/main/skills/engineering/tdd)
> skill (MIT, see LICENSE). The sixloom changes: the seams are fixed by the
> approved contract instead of being agreed per session, the loop is driven
> by locked acceptance tests, and the rules for locked tests are added.
> [tests.md](tests.md) and [mocking.md](mocking.md) are upstream, unchanged.

TDD is the red → green loop. This skill is the reference that makes that
loop produce tests worth keeping. Every section applies on every cycle.

Read `GLOSSARY.md` or the spec's own words first, so that test names and
interface vocabulary match the domain language. Respect the ADRs in
`system-design.mdx`.

## What a good test is

Tests verify behavior through public interfaces, not implementation
details. Code can change entirely; tests shouldn't. A good test reads like a
specification: "user can checkout with valid cart" tells you exactly what
capability exists, and it survives refactors because it doesn't care about
internal structure.

See [tests.md](tests.md) for examples and [mocking.md](mocking.md) for
mocking guidelines. Mock at system boundaries only (external APIs, time,
randomness), never your own modules.

## Seams: where tests go

A **seam** is the public boundary you test at. In a sixloom project the
seams are **already agreed**, because the humans approved them at the gates:

| Seam | Defined by | Tests |
|---|---|---|
| The UI as a user sees it | `SCREENS.md` and the preview | locked e2e |
| The HTTP API | `contracts/openapi.yaml` | locked contract and integration |
| Service or module boundaries inside an app | ADRs in `system-design.mdx` | unit, written in the loop |

Unit tests go at the **highest internal seam** that isolates the logic
(usually the service function an operation calls), never at private
helpers. If a new internal seam is needed, put it at the highest point you
can and note it in the PR body. You do not need to ask, because it does not
change any approved contract.

## The loop for one slice

1. **Outer red**: pick one red locked test for the slice (contract first,
   then e2e).
2. **Inner red**: write one unit test at the internal seam for the next
   smallest piece of behavior that test needs. Run it and watch it fail for
   the right reason.
3. **Green**: write only enough code to pass it. Don't anticipate future
   tests.
4. Repeat 2–3 until the outer test passes. Then take the next outer test.
5. Refactoring is not part of the loop. Do it once the slice is green, with
   every test still passing, and before opening the PR.

## Anti-patterns

- **Implementation-coupled**: mocks internal collaborators, tests private
  methods, or verifies through a side channel (querying the database instead
  of the API). The tell: it breaks on refactor while behavior is unchanged.
- **Tautological**: the expected value is recomputed the way the code
  computes it. Expected values come from an independent source: the spec,
  the contract's `examples`, or a known literal.
- **Horizontal slicing**: writing all unit tests first, then all code.
  Work one test → one implementation → repeat. (The locked acceptance suite
  is the deliberate exception. It was written up front from the spec and
  proven against the preview, so it checks agreed behavior, not imagined
  behavior.)

## Locked tests: strict rules

A file listed in `.sixloom/tests.lock` is a contract with the humans.

- **Allowed**: fix a selector or a wait, make setup deterministic, add new
  tests.
- **Forbidden**: delete or rename a locked test; change, weaken or remove an
  assertion; add `skip`, `only`, `fixme` or `fail`; inflate a timeout to hide
  slowness.
- If a locked test seems wrong, stop. Raise an **sxl-patch** change record of
  kind `test` and continue with another slice. `sixloom tests verify` runs in
  CI and will fail the PR anyway.
