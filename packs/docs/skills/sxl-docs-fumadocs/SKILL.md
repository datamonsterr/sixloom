---
name: sxl-docs-fumadocs
description: Scaffold and maintain a sixloom docs site with Fumadocs (Next.js, GitBook-like). It covers MDX pages, Mermaid rendered to static SVG at build time, and API reference pages generated from contracts/openapi.yaml. Use when sixloom.toml has docs.engine = "fumadocs", during the sxl-api-docs loom, or when the user asks to set up or fix the Fumadocs docs site.
---

# sxl-docs-fumadocs

Verified on create-fumadocs-app 16.2.26, fumadocs-core 16.17, fumadocs-mdx 15.4,
fumadocs-openapi 12.5, Next 16.4 (October 2026). If `npm view` shows a newer
major version, read its changelog before you copy the snippets below.

Layout: the site lives in `docs/`, pages live in `docs/content/docs/`, and the
contract is `../contracts/openapi.yaml` relative to `docs/`.

## 1. Scaffold (from the repo root)

`docs/` usually exists already, holding the data-model and system-design
pages and `docs/diagrams/`. The scaffolder refuses a non-empty directory, so
move it aside first and restore it into the content directory afterwards:

```bash
mv docs .docs-pages   # then scaffold as below, then:
# cp -r .docs-pages/<old content dir>/. docs/<content dir>/ && cp -r .docs-pages/diagrams docs/ && rm -r .docs-pages
```

```bash
npx -y create-fumadocs-app@latest docs --template +next+fuma-docs-mdx --pm npm \
  --install --no-git --linter biome --search orama -y
cd docs && npm i fumadocs-openapi shiki beautiful-mermaid
```

The current template defines collections in `lib/source.ts` with the macro
API (`defineDocs` from `fumadocs-mdx/macro`) and has no `source.config.ts`.
You add one only for MDX options.

## 2. Wire it up (copy from [files/](files/))

| File | What it does |
|---|---|
| `source.config.ts` | adds `remarkMdxMermaid`, so ```mermaid fences become `<Mermaid chart>` |
| `components/mdx/mermaid.tsx` | server component that renders Mermaid to **static SVG** with `beautiful-mermaid` (no browser, works in the built HTML), falling back to a code block |
| `components/mdx.tsx` | registers `Mermaid` with the default MDX components |
| `lib/openapi.ts` | `createOpenAPI({ input: ['../contracts/openapi.yaml'] })` |
| `components/api-page.tsx` | client `createOpenAPIPage()` |
| `lib/source.ts` | adds `openapi: await openapi.staticSource({ baseDir: 'api-reference' })` to the loader, plus `openapi.loaderPlugin()`, and handles openapi pages in `llms` |

Then make two edits:

- In `app/docs/[[...slug]]/page.tsx`, right after `if (!page) notFound();`, add:
  ```tsx
  if (page.type === 'openapi') {
    return (
      <DocsPage full>
        <DocsTitle>{page.data.title}</DocsTitle>
        <DocsDescription>{page.data.description}</DocsDescription>
        <DocsBody><OpenAPIPage {...page.data.getOpenAPIPageProps()} /></DocsBody>
      </DocsPage>
    );
  }
  ```
  (`import { OpenAPIPage } from '@/components/api-page';`). Adding openapi
  pages makes the page type a union, so guard every `page.data.body` use
  with `page.type`.
- In `app/global.css`, add `@import 'fumadocs-openapi/css/preset.css';` after
  the fumadocs-ui preset.

API pages are virtual (one per `operationId`, at
`/docs/api-reference/<operationId>`). They are rebuilt from the contract on
every build and in dev, so there is no command to regenerate them.

## 3. Sidebar order (`meta.json`)

```jsonc
// content/docs/meta.json
{ "title": "Docs", "pages": ["index", "getting-started", "architecture", "api-reference", "codebase-tour", "testing", "changelog"] }
// content/docs/architecture/meta.json
{ "title": "Architecture", "pages": ["system-design", "data-model"] }
// content/docs/api-reference/meta.json  (a real file next to the virtual pages)
{ "title": "API reference", "pages": ["<operationId>", "..."] }
```

Do not also pass `meta: true` to `staticSource`.

## 4. Commands

Add these to the `justfile`: `docs` → `cd docs && npm run dev`, and
`docs-build` → `cd docs && npm run build && npm run types:check`.
A clean build takes about 30 seconds.

## Check before the gate

- `npm run build` passes. The built `.next/server/app/docs/architecture/data-model.html`
  contains `<svg`, which means Mermaid rendered.
- Every operation has a page, and its `sxl:` marker line shows in the
  description.
- Search (Orama) finds an AC ID: `curl 'localhost:3000/api/search?query=F-001.AC1'`
  returns the pages that carry it.

## Gotchas

- `next dev` writes an `AGENTS.md` into `docs/`. Set `agentRules: false` in
  `next.config` to stop it, so it doesn't clash with the repo's own instructions.
- Rename the template defaults in `lib/shared.ts` (`appName = 'My App'`, `gitConfig` pointing at fumadocs)
  to the product's name and repo.
- Set `metadataBase` in the root layout metadata to silence the build warning.
- The two CSS `::highlight` warnings during optimization are harmless.
