# Mock scenario pattern (MSW 2 and 3)

This pattern is framework-agnostic. For Next.js or Nuxt, start the worker in
a client-only entry point.

```ts
// src/mocks/scenarios.ts
export type Scenario = 'ready' | 'empty' | 'error' | 'slow' | 'partial' | 'forbidden' | 'offline';

export function currentScenario(): Scenario {
  const url = new URL(window.location.href);
  return (url.searchParams.get('sxl') as Scenario) ??
    (sessionStorage.getItem('sxl') as Scenario) ?? 'ready';
}
```

```ts
// src/mocks/handlers.ts
import { http, HttpResponse, delay } from 'msw';
import { currentScenario } from './scenarios';
import { db } from './fixtures/db';          // in-memory, seeded from fixtures

const latency = { list: 700, detail: 400, write: 300 };

export const handlers = [
  http.get('/api/projects', async () => {
    const s = currentScenario();
    await delay(s === 'slow' ? 3000 : latency.list);
    if (s === 'offline') return HttpResponse.error();
    if (s === 'error') return HttpResponse.json({ code: 'internal', message: 'Something went wrong' }, { status: 500 });
    if (s === 'forbidden') return HttpResponse.json({ code: 'forbidden', message: 'No access' }, { status: 403 });
    return HttpResponse.json({ items: s === 'empty' ? [] : db.projects.all(), next: null });
  }),

  http.post('/api/projects', async ({ request }) => {
    await delay(latency.write);
    const body = (await request.json()) as { name?: string };
    if (!body.name?.trim())
      return HttpResponse.json(
        { code: 'validation', message: 'Invalid input', fields: { name: 'Name is required' } },
        { status: 422 },
      );
    return HttpResponse.json(db.projects.create(body), { status: 201 });
  }),
];
```

```ts
// src/main.tsx (or equivalent entry)
async function enableMocks() {
  if (import.meta.env.VITE_API_MODE !== 'mock') return;
  const { worker } = await import('./mocks/browser');
  // MSW 3 renamed onUnhandledRequest to onUnhandledFrame; check `npm ls msw`.
  await worker.start({ onUnhandledFrame: 'bypass', quiet: true });
}
enableMocks().then(() => mountApp());
```

Error bodies use one envelope: `{ code, message, fields? }`. The API loom
adopts the same envelope in `openapi.yaml`, so the UI error handling does not
change when the real backend arrives.

The same handlers run in Node for component tests (`msw/node`) and in
Playwright, where the preview server runs in mock mode, so one set of mocks
serves the preview, the tests and the e2e proof run.
