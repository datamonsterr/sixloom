---
name: sxl-specify
description: Write or update spec/SPEC.md, the human-confirmed list of features with stable IDs and acceptance criteria. This is the first sixloom loom. Use after `sixloom init`, when the user says "specify", "write the spec", "/sxl-specify", or brings user stories or use cases for a product to build. With --from-code, reverse-engineer a first spec from an existing codebase.
---

# sxl-specify: the spec loom

Follow the **sxl-loom** protocol (start, iterate, gate, end). Loom ID: `spec`.

## Inputs

- What the user brings: problem statement, user stories, use-case
  specifications, notes from product discovery. Ask for them if none were
  given. Discovery is assumed done, so do not run discovery again. Your job is
  to turn its output into features.
- `--from-code`: read the routes, pages, models and README of the existing
  repo. Write the spec of what the code does today, and mark the spec
  `origin: reverse-engineered`.

## Output: `spec/SPEC.md`

Copy [SPEC.template.md](SPEC.template.md) and fill it in. The spec holds
**only what a human must understand and confirm**:

- **Keep**: who uses the product, the features, what each feature lets the
  user do, the acceptance criteria, what is out of scope, and the open
  decisions.
- **Leave out**: tech stack, file paths, database or endpoint names, and UI
  layout. Those belong to later looms, and putting them here makes the human
  review things they cannot judge.

### Writing features

- One feature = one user-visible capability. ID `F-001`, `F-002`, … IDs
  are never reused or renumbered. A removed feature stays in the file with
  status `dropped`.
- Each feature lists the user stories it covers (`US-xx` from discovery,
  or written in "As a…, I want…, so that…" form).
- **Acceptance criteria** (`F-001.AC1`) use short Given / When / Then
  sentences in observable terms. Each AC becomes one e2e test later, so
  every AC must be checkable from the UI or the API. Include the unhappy
  paths: empty, invalid input, no permission, not found, network failure.
- Priority is `must`, `should` or `could`. Only `must` features gate the build.
- If two features cannot be told apart by their acceptance criteria, merge
  them.

### Round structure

1. Draft the whole spec from the inputs in one pass.
2. Ask the open questions with the `grilling` format: numbered, each with a
   recommended answer, at most five per round. Unanswered questions go to the
   spec's "Open decisions" section; they do not block the draft.
3. Revise. Repeat until the open decisions are either answered or explicitly
   deferred.

## Gate (product reviewer)

1. Does the feature list cover everything the product must do in v1, and
   nothing it must not?
2. Reading only the acceptance criteria, would you accept a product that
   satisfies them?
3. Are the `must` priorities right?
4. Is anything in "Out of scope" something you actually expect?
5. Are the remaining open decisions OK to defer?

Machine checks to run before the gate (`sixloom loom end spec` runs them too):
every feature has at least one AC, the IDs are unique and sequential, and no
AC is empty.

## Next

`sixloom design` (the sxl-design loom) in a new session. The product track
(design, then preview) finishes before the engineering track starts.
