---
name: sxl-data-model
description: The sixloom data model loom (data-first design). It derives entities, attributes, relationships, constraints and lifecycle states from spec/SPEC.md and the approved preview's data needs, and writes a reviewable data-model.mdx page with a rendered ER diagram, one table per entity, and a traceability table. Use after the preview is approved, when the user says "data model", "/sxl-data-model", "design the database", or asks for entities, an ERD or a schema.
---

# sxl-data-model: the data model loom

Follow the **sxl-loom** protocol. Loom ID: `data`. Run it in a new session.
The context pack is `spec/SPEC.md`, `design/SCREENS.md` and the approved
preview's data layer: each interface's API client and mocks, at the paths
`sixloom layout` shows. The preview's fixtures show exactly what data the
interfaces need, so start from them.

sixloom is **data-first**: the backend is designed from the data the
features need, not from endpoints or services. Get this right and the API and
services follow.

## Method

1. **Nouns and verbs.** Go through every AC and every mock fixture. The nouns that hold state are
   candidate entities, and the verbs that change state are lifecycle
   transitions. List both, each with the AC IDs it came from.
2. **Entities.** Merge synonyms using the spec's own words (a glossary of the
   domain language). Drop nouns that are only views of other data. Each entity
   gets a definition sentence that a product person agrees with.
3. **Attributes.** Give the minimal set the ACs need: type, nullability,
   default, uniqueness, and validation rules taken from the spec ("R-01: at
   most 5 active projects" becomes a constraint plus where it is enforced).
4. **Relationships.** Give the cardinality and what happens on delete
   (cascade, restrict or soft delete), and why.
5. **Lifecycle.** For each entity that has states, draw a Mermaid
   `stateDiagram-v2` with transitions labelled by AC ID.
6. **Access.** For each actor, what they can read and write
   (row-level rules). Say it in a sentence, not in a policy language.
7. **Volumes and hot paths.** Give a rough row count at one year and the
   queries the screens need most (from `SCREENS.md`, if it exists), with the
   indexes they need.

Do not choose the database engine here unless the spec forces it. That
belongs to the system loom. Types are logical (`text`, `int`, `money`,
`timestamp`, `enum`, `uuid`, `json`).

## Output

Write to the docs site content directory from `sixloom.toml`
(`docs.content_dir`, for example `docs/content/docs`), at
`architecture/data-model.mdx`, following [DATA-MODEL.template.mdx](DATA-MODEL.template.mdx):

- An overview Mermaid `erDiagram` covering all entities and relationships.
  It renders in GitHub and in all three supported docs engines.
- One section per entity: definition, attribute table, constraints,
  lifecycle diagram, access rules, and a trace marker under the heading,
  `<sub>sxl: F-002 · F-003.AC2</sub>`, naming the features it serves.
- A traceability table: feature → entities. Every `must` feature must appear.
- Decisions taken, as short ADR-style notes (context, decision, consequence).

If the docs site has not been scaffolded yet, write the file at that path
anyway. The api loom scaffolds the site around it.

## Gate (engineer reviewer)

1. Does every entity and attribute earn its place (traceable to an AC)?
2. Are the delete behaviors and uniqueness rules right?
3. Will the hot-path queries be fast with the listed indexes?
4. Are the access rules correct for every actor?
5. Is anything going to be painful to migrate later (enums, JSON blobs, money)?

## Next

`/sxl-system-design` in a new session.
