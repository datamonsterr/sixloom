---
name: sxl-build
description: The sixloom autonomous build loom. With the preview and the API docs as the goal and the locked test suites as the stop condition, it implements the product in vertical slices with TDD, runs parallel subagents in separate git worktrees, opens a PR per slice, waits for CI to go green, merges, and repeats until every locked test passes against the real backend. It then runs sxl-validate. Use when the tests loom is approved, or when the user says "/sxl-build", "build it", "implement everything", or "run the autonomous loop".
---

# sxl-build: the autonomous build loom

Follow the **sxl-loom** protocol. Loom ID: `build`. The context pack is
`spec/SPEC.md`, `contracts/openapi.yaml`, `system-design.mdx` (ADRs),
`data-model.mdx`, `.sixloom/tests.lock` and `sixloom.toml` `[build]`.

Code goes only into the interface and service folders recorded in
`sixloom layout`. A slice that needs a new service or folder records it
first with `sixloom layout service add …`, adds an ADR line to
`system-design.mdx`, and re-approves the system loom.

**The goal** (state it at the top of the session and in `.sixloom/runs/build.md`):

> The real app behaves exactly like the preview and exactly as the API
> docs state. This is proven by every locked e2e, contract and integration
> test passing in real mode, and by `sxl-validate` finding no gaps.

## 1. Plan slices

1. Run `sixloom trace --json`. Each `must` feature (then `should`) is a
   **slice**: its contract operations, its e2e ACs, and the entities it
   writes.
2. Order the slices with the data model as a dependency graph. A slice
   depends on the slices that create the entities it reads. Slices with no
   dependency between them and **disjoint write sets** may run in parallel.
3. Write the plan to `.sixloom/runs/build.md` as a checklist: slice, depends
   on, tests, status. This file is the build's memory. Re-read it after every
   merge, not the whole repo.

## 2. Run slices

For each ready slice, up to `build.max_parallel` at once (default 3):

- **Subagents available** (Claude Code Task or agents, Codex subagents, Kimi
  subagents): spawn one per slice with a self-contained brief covering the
  slice ID, the AC and operation IDs, the files likely to change, the rules
  below, and the instruction to work in
  `git worktree add ../<repo>-<slice> -b sxl/<slice>`.
- **No subagents**: run the slices one at a time yourself.

Inside a slice, follow **sxl-tdd**:

1. Pick the next red contract or e2e test for the slice.
2. Write the unit test at the seam (service or module boundary) that will
   make it pass, and watch it fail.
3. Write the minimal code to pass it. Repeat until the slice's locked tests
   are green in real mode (`just contract`, `just e2e -g "F-003"`).
4. Run `just ci-fast` locally. **Never push red.**
5. Commit with `feat(F-003): …`, open a PR titled `F-003 <feature>` with the
   AC checklist in the body, and wait for CI.
6. CI red: reproduce it with the `just` command the job prints, fix it, and
   push. CI green: merge (squash, or the merge queue if the repo has one).
   Then rebase the other open slice branches.

## 3. Rules for tests (strict)

| You may | You may not |
|---|---|
| add new tests of any tier | delete or rename a locked test |
| fix a locked test's **selector or wait**, keeping its assertions | change, weaken or remove an assertion in a locked test |
| add unit tests freely, and refactor them | add `skip`, `only`, `fixme` or `fail`, or raise timeouts to hide slowness |
| make a test deterministic (seed, clock) | make a test pass by changing mocks in real mode |

`sixloom tests verify` enforces the right-hand column in CI. If a locked
test looks wrong (it contradicts the spec, or the spec is ambiguous), **stop
that slice**. Write a change record with **sxl-patch** (status `proposed`),
mark the slice `blocked` in `build.md`, and continue with the other slices.
Never "fix" the test yourself.

## 4. Budget and escalation

- `build.max_attempts_per_slice` (default 3): a red → fix → red cycle on the
  same failure counts as one attempt. At the limit, mark the slice `blocked`
  with the failing command, the error, what you tried, and your best
  hypothesis (use the `diagnosing-bugs` skill), then move on.
- No GitHub access, or `build.pr = false`: merge locally after `just ci`
  passes, one slice at a time.
- At the end, list the blocked slices for the human. A blocked slice is
  information, not a failure to hide.

## 5. Finish

When every slice is merged or blocked:

1. Run the full `just ci` in real mode on the main branch.
2. Run **sxl-validate**, and fix its findings as new slices (same rules).
3. Run `sixloom loom end build`, and write the run summary at the end of
   `.sixloom/runs/build.md`: slices done or blocked, PRs, CI time, tests
   added, and change records raised.

## Gate

None. The build is autonomous. The human reviews the **validate** report.
