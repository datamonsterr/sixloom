---
name: sxl-design
description: The sixloom UI design loom. It turns spec/SPEC.md into a design system (design/DESIGN.md), a complete screen × state inventory with user flows (design/SCREENS.md), and visual comps of the key screens made with Claude Design, Google Stitch, or plain HTML. Use after the spec is approved, when the user says "design the UI", "/sxl-design", or asks for screens, flows or a design system.
---

# sxl-design: the UI design loom

Follow the **sxl-loom** protocol. Loom ID: `design`. Run it in a **new
session**. The context pack is `spec/SPEC.md`, plus the team's existing
design system if there is one.

`sixloom.toml` sets `design.tool` (`claude`, `stitch` or `none`) and
`design.fidelity` (`key`, `all` or `none`).

## Step 1: Screen inventory (`design/SCREENS.md`)

Copy [SCREENS.template.md](SCREENS.template.md). This file matters most,
because the preview, the e2e tests and the final visual check all come from
it.

- **Every screen** the spec needs: routes, modals, drawers, empty
  first-run views, auth screens, settings, 404, and the global error page.
- **Every functional control** on each screen: buttons, links, inputs, menu
  items. Note the AC each one serves (`F-002.AC1`). A control with no AC is a
  question for the human, not an invention.
- **Every state** of each screen. These are mandatory where they apply:
  `loading` (skeleton, not a spinner, for content areas), `empty`, `ready`,
  `error` (with retry), `partial` (some data failed), `submitting`,
  `validation-error`, `no-permission`, `offline`. Say what triggers each one
  and what the user can do from it.
- **Flows** as Mermaid `flowchart` diagrams, one per key flow in the spec,
  with screens as nodes and actions as edges.

Run the coverage check: every AC in the spec maps to at least one
screen/state/control, and every screen maps to at least one feature.
`sixloom loom check design` reports the ACs that appear on no screen.

## Step 2: Design system (`design/DESIGN.md`)

If the team gave a design system, record its tokens and rules here and stop.
Otherwise, copy [DESIGN.template.md](DESIGN.template.md) and make deliberate,
subject-specific choices. Load the `frontend-design` skill for aesthetic
direction. Give tokens (color, type scale, spacing, radius, elevation,
motion), component rules, and the skeleton and error patterns, so the
preview never has to guess.

## Step 3: Comps (according to `design.fidelity`)

- `key` (default): comps for the 3–5 screens that set the visual language
  (usually the main list, a detail view, a form, and one empty or error state).
  The preview builds the rest from `DESIGN.md`.
- `all`: comps for every screen in the ready state.
- `none`: no comps. Go straight to the preview.

By `design.tool`:

- **claude**: design with Claude Design. Each comp is saved in the repo as
  `design/exports/<screen-id>.dc.html`, the Claude Design file format, which
  renders again when it is reopened. Without Claude Design, write a plain
  self-contained HTML comp with the `DESIGN.md` tokens to
  `design/exports/<screen-id>.html`.
- **stitch**: use the **stitch** skill (Google's `stitch` CLI, from
  `@google/stitch`). First check `stitch status --json`. If the user is not
  signed in, ask them to run `stitch login` once (browser OAuth). That is
  the one step that needs them. Then create or bind the project
  (`stitch create project`, `stitch config set project <id>`, after
  confirming with the user), generate each comp screen with
  `stitch generate screen` from a prompt built from its `SCREENS.md` entry
  and `DESIGN.md`, and **pull the result back into the repo**: save each
  screen's HTML to `design/exports/<screen-id>.html` (use
  `stitch get screen <id> --project <p> --json` for its HTML or download URL;
  `stitch get screen --schema` shows the fields). Record the Stitch project
  and screen IDs in `design/exports/stitch.json`. `DESIGN.md` and
  `SCREENS.md` stay the source of truth. Stitch is where the comps are made,
  not where the decisions live.
- **none**: skip.

## Gate (product reviewer)

1. Walking through each flow diagram, is anything missing or in the wrong
   order?
2. Does every screen have the buttons you expect, and no extras?
3. Are the loading, empty and error states what you want users to see?
4. Do the comps look like *your* product (tone, color, type)?
5. Is there any screen that you would want to see as a comp before the
   preview?

Then run `sixloom loom end design`.

## Next

`/sxl-preview` in a new session.
