// sixloom template, verified on a two-feature example app (Tasklet). Keep the structure;
// replace screen IDs, seeds, scenarios and endpoints with your project's.
import { readFileSync } from 'node:fs';
import { parse } from 'yaml';
import Ajv from 'ajv/dist/2020.js';
import addFormats from 'ajv-formats';

const spec = parse(readFileSync(new URL('../../contracts/openapi.yaml', import.meta.url), 'utf8'));
const ajv = new Ajv({ strict: false });
addFormats(ajv);
ajv.addSchema({ $id: 'c', components: spec.components });

/** Validate a body against the documented response; returns ajv errors or null. */
export function conforms(path: string, method: string, status: number, body: unknown) {
  let r = spec.paths[path]?.[method]?.responses?.[String(status)];
  if (!r) return [`${method.toUpperCase()} ${path} does not document ${status}`];
  if (r.$ref) r = spec.components.responses[r.$ref.split('/').pop()];
  const schema = JSON.parse(JSON.stringify(r.content['application/json'].schema).replaceAll('#/components', 'c#/components'));
  const v = ajv.compile(schema);
  return v(body) ? null : v.errors;
}
