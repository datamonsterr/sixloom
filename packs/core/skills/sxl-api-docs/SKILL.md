---
name: sxl-api-docs
description: The sixloom API and documentation loom. It writes the OpenAPI contract (contracts/openapi.yaml) from the spec, data model and system design, reconciles it with the preview's mocks, and scaffolds a GitBook-style docs site (Fumadocs, Docusaurus or Starlight) with the API reference, architecture pages, install guide and codebase tour that a newcomer reads first. Use after the system design is approved, when the user says "API docs", "/sxl-api-docs", "documentation site", or asks for an OpenAPI spec.
---

# sxl-api-docs: the API and docs loom

Follow the **sxl-loom** protocol. Loom ID: `api`. Run it in a new session.
The context pack is `spec/SPEC.md`, `design/SCREENS.md`, `data-model.mdx`,
`system-design.mdx`, and each interface's API client and mocks (paths from
`sixloom layout --json`).

## Step 1: The contract (`contracts/openapi.yaml`, OpenAPI 3.1)

- Start from the approved preview: every method of every interface's API
  client, and every mock handler, becomes an operation, and its fixtures
  become the schema `example`s. The preview is final, so the contract follows it, not the other
  way round. Check that the data model can serve every one of them.
- Each operation has an `operationId`, a summary, and a `description` whose
  last line is the trace marker `sxl: F-003.AC1, F-003.AC2`. The API reference
  renders the marker, so pasting an ID into the docs search finds the
  endpoint. The IDs never appear in paths, parameters or payloads.
- Schemas reuse the data model's names and rules (lengths, enums, formats).
- Use one error envelope everywhere: `{ code, message, fields? }`, with
  documented `code` values per operation. 422 is for validation, 409 for
  conflicts, 403 for permission, 404 for not found.
- Specify pagination, sorting, filtering and auth once, in components, and
  reference them.
- Include an `example` on every request and response schema. The examples
  double as mock fixtures and test data.
- A public API declares `security: []` at the root explicitly, and every
  API declares `info.license`. Without them, Redocly's default rules fail.
- Validate with `npx @redocly/cli lint contracts/openapi.yaml`, with no errors
  allowed.

## Step 2: Reconcile with the preview (the contract gate)

1. Generate each interface's client types from the contract, and switch the
   client to them:
   - **TypeScript** (web, Ink, Expo, Electron): add a script
     `"gen:api": "npx -y openapi-typescript@7 <contract path relative to the interface> -o <api_client>/schema.d.ts"`.
     Run it through `npx`, not as a devDependency, because its peer range
     lags behind new TypeScript majors.
   - **Rust**: `progenitor` or `openapi-generator` (rust); **Go**: `oapi-codegen`;
     **Python**: `datamodel-code-generator`; **Dart**: `openapi-generator` (dart-dio).
   - Error codes that only the client produces (for example `offline`) stay a
     client-side union, not contract enums.
2. Build or typecheck each interface. Then run the **contract gate**: every
   mock response must validate against the contract.
   - **Web with MSW**: copy [check-mocks.template.mjs](check-mocks.template.mjs)
     to `scripts/check-mocks.mjs` and point it at the interface's mocks. It
     runs the real handlers through `msw/node`.
   - **Other kinds**: validate the fixture files in the interface's mocks
     folder against the matching response schemas, using the same ajv approach
     on the JSON or YAML fixtures.
   List one case per documented status, and add the gate to the justfile as
   `check-mocks`, which is part of `ci-fast`. The gate fails on undocumented
   statuses, so it can never pass vacuously.
3. A mismatch almost always means that the contract is wrong, so fix it.
   Change the approved preview only when its data cannot be served (for
   example, a field the data model cannot provide). That change goes through
   **sxl-patch** (kind `sync`), and it updates the spec as well. List every
   decision in the gate.

## Step 3: The docs site

The engine comes from `sixloom.toml` `docs.engine`. Load the matching skill:
`sxl-docs-fumadocs`, `sxl-docs-docusaurus` or `sxl-docs-starlight`. It
scaffolds `docs/`, wires Mermaid, and generates the API reference pages from
`contracts/openapi.yaml`.

`docs/` already holds the data-model and system-design pages and
`docs/diagrams/`, and every scaffolder refuses a non-empty directory. So:
`mv docs .docs-pages`, scaffold, copy `.docs-pages/<content>/.` into the new
content directory and `.docs-pages/diagrams` into `docs/`, then delete
`.docs-pages`. Rename the template's site title and repo links to the
product's.

Required pages, in this sidebar order:

1. **Introduction**: what the product is (from `SPEC.md` "Why"), with a link
   to the preview.
2. **Getting started**: prerequisites, install, `just dev`, environment
   variables (names and purpose only, never values), and the seed data.
3. **Architecture**: system design, data model (already written), and
   decisions.
4. **API reference**: generated from the contract. Never hand-write it.
5. **Codebase tour**: the folder map (see the sixloom layout in
   `AGENTS.md`), where each kind of change goes, and the loom workflow.
6. **Testing**: test tiers, how to run them, and the locked-test rules.
7. **Changelog**: rendered from `spec/CHANGELOG.md`.

Pages are `.mdx`. Keep prose short. A newcomer should be able to read the
whole site in 30 minutes.

This loom also changes the interfaces (types, fixtures) and adds `scripts/`,
so end it with `sixloom loom end api --all`.

## Gate (engineer reviewer)

Run `just docs`, then ask:

1. Reading only the API reference, could a front-end developer build every
   screen?
2. Are the error codes and status codes right for each operation?
3. Are the pagination, auth and naming conventions what you want?
4. Do you accept each contract-versus-preview reconciliation decision?
5. Could a newcomer set up and run the project from "Getting started" alone?

## Next

`/sxl-test-suites` in a new session (needs the `preview` loom approved too).
