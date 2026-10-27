---
name: sxl-validate
description: Final sixloom validation. It checks the working product against the goal (the preview and the API docs), using the full locked suite in real mode, a preview-versus-real visual comparison of every screen state, trace coverage, and an exploratory pass for gaps, then reports the findings as change records or bug slices. Use at the end of sxl-build, before a release, or when the user says "/sxl-validate", "are we done", or "find gaps".
---

# sxl-validate: is it what we said it would be?

Follow the **sxl-loom** protocol. Loom ID: `validate`. Write the report to
`.sixloom/runs/validate-<date>.md`, starting with frontmatter
`result: pass` or `result: fail`. `sixloom done` reads it.

## Checks

1. **Locked suite, real mode.** `just ci` on the main branch, with real
   backend, seeded database and `E2E_MODE=real`. Every locked test must pass.
2. **Trace.** `sixloom trace` must show zero `must` ACs without a passing
   test, and zero operations in the contract without an implementation (no
   remaining 501).
3. **Preview versus real (visual).** For each screen × state in
   `apps/web/PREVIEW.md` that is reachable in real mode (ready, empty,
   validation-error, and the others you can seed), take the screenshot in
   real mode and compare it with the preview screenshot
   (`apps/web/.preview-shots/`). Use Playwright `toHaveScreenshot` with a
   `maxDiffPixelRatio` of `validate.visual_tolerance` (default 0.005).
   Differences caused only by data are fine; layout, missing elements and
   wrong states are findings.
4. **Docs truth.** The OpenAPI contract test passes against the running API.
   Every "Getting started" command in the docs runs as written on a clean
   checkout.
5. **Quality floor.** In real mode, every screen state has no console errors
   and no serious or critical axe accessibility violations
   (`@axe-core/playwright`). CI is green on the main branch for the commit
   being validated. No slice is still `blocked` in `.sixloom/runs/build.md`.
   No change record is still `proposed`.
6. **Exploratory pass.** Walk each key flow from `SPEC.md` in the real app
   with `playwright-cli`, the way a first-time user would. Try the unhappy
   paths the ACs do not mention: double submit, back button, refresh
   mid-flow, long text, slow network. Each surprise is a finding.

## Findings

For each finding: give the evidence (command, screenshot or trace), the
affected IDs, and the kind. A **bug** means an existing AC is violated: it
goes back to sxl-build as a slice. A **gap** means behavior the spec never
covered: draft an sxl-patch record with status `proposed`.

## Gate: the user says it is done

Run `sixloom done`. It checks the machine conditions: every loom approved
and none stale, locked tests intact, a complete trace, an empty changelog
stack, and a validate report with `result: pass`. It then lists the
conditions only a human can confirm. Show both lists, and ask:

1. Do you accept the product as matching the preview and the docs?
2. Which proposed changes go into this version, and which go into the next?

**Done is the user's call.** Only after an explicit "yes, it's done", run
`sixloom done --confirm`, which records their sign-off. Then say that new
feedback starts the next cycle through **sxl-patch**.
