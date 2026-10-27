---
name: sxl-docs-docusaurus
description: Scaffold and maintain a sixloom docs site with Docusaurus 3. It covers MDX pages, Mermaid through @docusaurus/theme-mermaid, and API reference pages generated from contracts/openapi.yaml by docusaurus-plugin-openapi-docs. Use when sixloom.toml has docs.engine = "docusaurus", during the sxl-api-docs loom, or when the user asks to set up or fix the Docusaurus docs site.
---

# sxl-docs-docusaurus

Verified on Docusaurus 3.10.2 and docusaurus-plugin-openapi-docs 5.2.0
(October 2026).

Layout: the site is `docs/`, pages are in `docs/docs/`, generated API pages
go to `docs/docs/api-reference/`, and the contract is
`../contracts/openapi.yaml` relative to `docs/`.

## 1. Scaffold (from the repo root)

`docs/` usually exists already, holding the data-model and system-design
pages and `docs/diagrams/`. The scaffolder refuses a non-empty directory, so
move it aside first and restore it into the content directory afterwards:

```bash
mv docs .docs-pages   # then scaffold as below, then:
# cp -r .docs-pages/<old content dir>/. docs/<content dir>/ && cp -r .docs-pages/diagrams docs/ && rm -r .docs-pages
```

```bash
npx -y create-docusaurus@latest docs classic --typescript --package-manager npm --skip-install
cd docs && npm install
npm i docusaurus-plugin-openapi-docs docusaurus-theme-openapi-docs @docusaurus/theme-mermaid docusaurus-plugin-sass sass
```

`docusaurus-plugin-sass` is a peer dependency of the OpenAPI theme. Then
delete the template's `docs/intro.mdx`, `docs/tutorial-*` and `blog/`. Give
`docs/index.mdx` `slug: /`, and change `to="/docs/intro"` to `to="/docs"` in
`src/pages/index.tsx`. Otherwise `onBrokenLinks: 'throw'` fails the build.

## 2. `docusaurus.config.ts`

```ts
import type * as OpenApiPlugin from 'docusaurus-plugin-openapi-docs';

// inside the config object:
markdown: { mermaid: true },
presets: [['classic', {
  docs: { sidebarPath: './sidebars.ts', docItemComponent: '@theme/ApiItem' }, // required for *.api.mdx
  blog: false,
  theme: { customCss: './src/css/custom.css' },
} satisfies Preset.Options]],
plugins: [['docusaurus-plugin-openapi-docs', {
  id: 'api', docsPluginId: 'classic',
  config: { api: {
    specPath: '../contracts/openapi.yaml',     // relative to docs/
    outputDir: 'docs/api-reference',
    sidebarOptions: { groupPathsBy: 'tag', categoryLinkSource: 'tag' },
    showSchemas: false,
  } satisfies OpenApiPlugin.Options },
}]],
themes: ['docusaurus-theme-openapi-docs', '@docusaurus/theme-mermaid'],
themeConfig: {
  mermaid: { theme: { light: 'neutral', dark: 'dark' } },
  navbar: { items: [{ type: 'docSidebar', sidebarId: 'docsSidebar', position: 'left', label: 'Docs' }] },
},
```

## 3. Sidebar order (`sidebars.ts`)

```ts
import type { SidebarsConfig } from '@docusaurus/plugin-content-docs';
import apiSidebar from './docs/api-reference/sidebar'; // generated

const sidebars: SidebarsConfig = {
  docsSidebar: [
    { type: 'doc', id: 'index', label: 'Introduction' },
    'getting-started',
    { type: 'category', label: 'Architecture', collapsed: false, items: ['architecture/system-design', 'architecture/data-model'] },
    { type: 'category', label: 'API reference', items: apiSidebar },
    'codebase-tour',
    'testing',
    'changelog',
  ],
};
export default sidebars;
```

## 4. API pages are generated files

The build does **not** regenerate them, so add these scripts to `package.json`:

```json
"gen-api": "docusaurus clean-api-docs all && docusaurus gen-api-docs all",
"prebuild": "npm run gen-api"
```

Regenerate after every contract change. Doc IDs are kebab-case operationIds
(`list-tasks`). `sidebars.ts` imports the generated `sidebar.ts`, so the build
fails until `gen-api` has run. The prebuild script handles that.

## 5. Commands

Add these to the `justfile`: `docs` → `cd docs && npm start`, and
`docs-build` → `cd docs && npm run build && npm run typecheck`. A build takes
about 13 seconds.

## Check before the gate

- `npm run build` passes.
- Mermaid renders **client-side**: the built HTML has no SVG, so check it in
  the browser (`npm run serve`) or with Playwright.
- Every operation has a page, and the `sxl:` marker shows in its description.
  Search the built site (local search plugin, or Algolia if configured) for an
  AC ID.
