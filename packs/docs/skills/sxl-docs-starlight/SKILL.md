---
name: sxl-docs-starlight
description: Scaffold and maintain a sixloom docs site with Astro Starlight. It covers MDX pages, Mermaid through astro-mermaid, built-in Pagefind search, and API reference pages generated from contracts/openapi.yaml by starlight-openapi. Use when sixloom.toml has docs.engine = "starlight", during the sxl-api-docs loom, or when the user asks to set up or fix the Starlight docs site.
---

# sxl-docs-starlight

Verified on Astro 7.4, @astrojs/starlight 0.42, starlight-openapi 0.27 and
astro-mermaid 2.2 (October 2026). Astro 7 wants Node ≥ 22.23; older 22.x
only warns.

Layout: the site is `docs/`, pages are in `docs/src/content/docs/`, and the
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
npm create -y astro@latest -- docs --template starlight --install --no-git --yes --skip-houston
cd docs && npm i starlight-openapi astro-mermaid mermaid
```

If the template download fails (create-astro fetches from
`codeload.github.com`, which some proxies block), copy the template with git
instead:

```bash
git clone --depth 1 --filter=blob:none --sparse https://github.com/withastro/starlight _tpl
(cd _tpl && git sparse-checkout set examples/basics)
cp -r _tpl/examples/basics docs && rm -rf _tpl
cd docs && npm pkg set name=docs && npm install && npm i starlight-openapi astro-mermaid mermaid
```

Replace the template's splash `index.mdx` and its `guides/` and
`reference/` folders with the sixloom pages.

## 2. `astro.config.mjs`

```js
// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import mermaid from 'astro-mermaid';
import starlightOpenAPI, { openAPISidebarGroups } from 'starlight-openapi';

export default defineConfig({
  site: 'https://docs.example.com',             // also silences the sitemap warning
  integrations: [
    mermaid({ theme: 'neutral', autoTheme: true }), // MUST come before starlight
    starlight({
      title: '<Product> docs',
      plugins: [starlightOpenAPI([{
        base: 'api-reference',
        schema: '../contracts/openapi.yaml',
        sidebar: { label: 'API reference', collapsed: false, operations: { badges: true } },
      }])],
      sidebar: [
        { label: 'Introduction', slug: 'index' },
        { label: 'Getting started', slug: 'getting-started' },
        { label: 'Architecture', items: [
          { label: 'System design', slug: 'architecture/system-design' },
          { label: 'Data model', slug: 'architecture/data-model' },
        ] },
        ...openAPISidebarGroups,
        { label: 'Codebase tour', slug: 'codebase-tour' },
        { label: 'Testing', slug: 'testing' },
        { label: 'Changelog', slug: 'changelog' },
      ],
    }),
  ],
});
```

The explicit `sidebar` array sets the order. Frontmatter `sidebar.order` only
applies inside `autogenerate` groups.

API pages (`/api-reference/operations/<operationid-lowercase>/`) are built
from the contract on every build and in dev, so there is no regenerate step.

## 3. Commands

Add these to the `justfile`: `docs` → `cd docs && npm run dev`, and
`docs-build` → `cd docs && npm run build`. A build takes about 8 seconds.

## Check before the gate

- `npm run build` passes.
- Mermaid renders **client-side**: the built HTML holds `<pre class="mermaid">`
  plus a loader script. No headless browser is needed at build time (unlike
  `rehype-mermaid`).
- Pagefind indexes the built pages, including the operation descriptions.
  Search an AC ID in `npm run preview` to confirm that the `sxl:` markers are
  findable.

## Gotchas

- Astro 7's `astro dev` can detach into a background daemon. Stop it with
  `npx astro dev stop`, and inspect it with `astro dev status` or
  `astro dev logs`.
- The warnings `collection "i18n" does not exist` and `Entry docs → 404 was not
  found` are harmless.
