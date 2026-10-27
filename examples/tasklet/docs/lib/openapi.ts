import { createOpenAPI } from 'fumadocs-openapi/server';

// Server-side only. Path is relative to the docs/ package (process.cwd()).
export const openapi = createOpenAPI({
  input: ['../contracts/openapi.yaml'],
});
