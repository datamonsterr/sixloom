---
name: sxl-patch
description: Turn feedback, a bug report, a spec–preview mismatch, or a wrong locked test into a sixloom change record on the changelog stack (spec/changes/CH-xxxx-*.md). It then updates the spec, the affected artifacts and the locked tests, in that order, before any code changes. Use when the user gives feedback on the preview or the working product, reports a bug, asks to change a feature, when another loom finds a mismatch, or when the user says "/sxl-patch".
---

# sxl-patch: feedback → patch → changelog stack

A patch never goes straight to code. The order is always:
**change record → spec → affected looms → locked tests → code**.
This order keeps the preview, the docs, the tests and the product telling the
same story.

## 1. Record

Run `sixloom change new "<short title>"`, or copy
[CHANGE.template.md](CHANGE.template.md) to `spec/changes/CH-<next>-<slug>.md`.
Fill in:

- **Kind**: `feedback`, `bug`, `sync` (artifacts disagree) or `test` (a locked
  test is wrong).
- **Source**: who or what raised it, with a quote or link.
- **Affects**: the feature and AC IDs. Run `sixloom trace F-003` to see the
  screens, operations and tests hanging off them.
- **Proposal**: the new or changed AC text, in spec language.
- **Invalidates**: the looms that must update (from the trace).

One change = one decision. Split a feedback list into several records.

## 2. Classify

- **Bug**: the product does not meet an existing AC. The spec is unchanged.
  Add a regression test if no locked test caught it (that test is a gap in
  itself, so note it), then fix it with sxl-tdd. Status goes straight to
  `accepted`.
- **Change**: the AC itself changes. The status stays `proposed` until a human
  runs `sixloom approve change CH-xxxx`. Ask with one clear yes or no question
  and a recommended answer.

## 3. Apply (accepted changes, oldest first)

1. **Spec**: edit `SPEC.md`. Changed ACs keep their ID, and the feature
   status becomes `changed`. New ACs get the next number. Removed ACs are
   struck through, never deleted. Status → `specced`.
2. **Looms**: `sixloom status` now shows the downstream looms as `stale`,
   naming the changed spec section (`spec/SPEC.md#F-001`). Use
   `sixloom trace F-001.AC2` to find what actually cites the change. Update only
   those parts (screen state, mock, endpoint, entity, test) with the
   respective sxl skill, then run `sixloom loom end <loom> --all --approve`.
   Re-approve the stale looms you checked and found unaffected together:
   `sixloom approve data system api`.
3. **Tests**: change or add the locked tests for the affected ACs, prove them
   on the preview (mock mode), then run `sixloom tests lock`. Changing a
   locked test's title or assertions is allowed **only** under an accepted
   change record, and `sixloom tests verify` fails until you re-lock. Status →
   `tested`.
4. **Code**: run the slice through sxl-build's loop. Status → `built`.

`spec/CHANGELOG.md` is regenerated from the records by `sixloom change log`.
Never edit it by hand.

## Gate

For `change` records only: the human approves the proposal (one question).
Everything after that is mechanical.
