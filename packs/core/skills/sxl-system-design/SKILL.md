---
name: sxl-system-design
description: The sixloom high-level system design loom. It lists the services and components, their responsibilities, the data stores and integrations, and how a request flows through them, with a Mermaid diagram as the diffable source and a draw.io file for humans. It records the stack decisions as ADRs. Use after the data model is approved, when the user says "system design", "architecture", "/sxl-system-design", or asks for a services diagram.
---

# sxl-system-design: the system design loom

Follow the **sxl-loom** protocol. Loom ID: `system`. Run it in a new
session. The context pack is `spec/SPEC.md`,
`<content>/architecture/data-model.mdx` and `sixloom.toml`.

Keep it **high level**: boxes and arrows a newcomer understands in five
minutes. Class diagrams and folder layouts do not belong here.

## Method

1. **Start with the smallest design that works.** For most v1 products that
   is one front end, one API service, one database, and the external services
   the spec forces (email, payments, storage, auth provider). Add a component
   only when an AC or rule needs it (a queue for "send the report within 5
   minutes", a cache for a hot path in the data model). Name the AC or rule.
2. **Responsibilities.** For each component, give one sentence on what it
   owns, plus the data entities it is the single writer for.
3. **Key flows.** Write one Mermaid `sequenceDiagram` per key flow in the
   spec, covering the request path and the failure path (what the user sees
   when each hop fails, matching the error states in `SCREENS.md`).
4. **Stack decisions (ADRs).** For language, framework, database, hosting,
   auth, and how tests run the backend locally (containers or an in-memory
   database), give one ADR each: context, options considered, decision,
   consequences. Prefer what the team already knows (ask once) and what
   runs in CI without paid services.
5. **Non-functional notes** from the spec only: data residency, retention,
   expected load. Do not invent SLOs the spec does not ask for.

## Output

`<content>/architecture/system-design.mdx` (see [SYSTEM.template.mdx](SYSTEM.template.mdx)) with:

- A Mermaid `flowchart` of the components. **This is the source of truth.**
- The components table, the sequence diagrams, and the ADRs. Each
  sequence diagram and each component gets a trace marker,
  `<sub>sxl: F-003</sub>`, naming the features it serves.

`docs/diagrams/system.drawio` is the human-editable view. Generate it from
the Mermaid source: draw.io imports Mermaid through *Arrange → Insert →
Advanced → Mermaid*. When writing the file directly, emit one `<mxCell>` per
node and edge with simple grid coordinates, using the same labels as the
Mermaid source. Keep the two in sync at every loom end. Writing draw.io XML by
hand costs many tokens, so update the Mermaid first and regenerate.

## Gate (engineer reviewer)

1. Is every component justified by a feature or rule?
2. Is the stack in the ADRs one the team can build and run?
3. Do the failure paths match the error states the preview shows?
4. Can the whole system run locally and in CI without paid services?
5. Is anything here you would remove for v1?

## Next

`/sxl-api-docs` in a new session.
