// sixloom template, verified on a two-feature example app (Tasklet). Keep the structure;
// replace screen IDs, seeds, scenarios and endpoints with your project's.
// Contract gate: every mock response must validate against contracts/openapi.yaml.
// Runs the real MSW handlers through msw/node, so it checks what the preview serves.
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
const require = createRequire(new URL('../apps/web/package.json', import.meta.url));
const { parse } = require('yaml');
const Ajv = require('ajv/dist/2020').default;
const addFormats = require('ajv-formats');
const { setupServer } = require('msw/node');

const spec = parse(readFileSync(new URL('../contracts/openapi.yaml', import.meta.url), 'utf8'));
const ajv = new Ajv({ strict: false });
addFormats(ajv);
ajv.addSchema({ $id: 'c', components: spec.components });
const ref = (s) => JSON.parse(JSON.stringify(s).replaceAll('#/components', 'c#/components'));
function responseSchema(path, method, status) {
  let r = spec.paths[path]?.[method]?.responses?.[status];
  if (!r) return null;
  if (r.$ref) r = spec.components.responses[r.$ref.split('/').pop()];
  return r.content?.['application/json']?.schema ?? null;
}

// Relative handler paths resolve against location, as in the browser.
globalThis.location = new URL('http://localhost/?sxl=ready');
globalThis.window = globalThis;
const { handlers } = await import('../apps/web/src/mocks/handlers.ts');
const server = setupServer(...handlers);
server.listen({ onUnhandledFrame: 'error' });

const cases = [
  ['/tasks', 'get', undefined],
  ['/tasks', 'post', { title: 'Water the plants' }],
  ['/tasks', 'post', { title: '' }],
];
let failed = 0;
for (const [path, method, body] of cases) {
  const res = await fetch('http://localhost/api' + path, {
    method: method.toUpperCase(),
    headers: { 'content-type': 'application/json' },
    body: body && JSON.stringify(body),
  });
  const schema = responseSchema(path, method, String(res.status));
  const label = `${method.toUpperCase()} ${path} → ${res.status}`;
  if (!schema) { console.error(`✗ ${label}: status not documented in the contract`); failed++; continue; }
  const v = ajv.compile(ref(schema));
  if (v(await res.json())) console.log(`✓ ${label}`);
  else { console.error(`✗ ${label}`, JSON.stringify(v.errors)); failed++; }
}
server.close();
console.log(`${cases.length - failed}/${cases.length} mock responses match the contract`);
process.exit(failed ? 1 : 0);
