import { Hono } from 'hono';
import { openStore } from './store.ts';

type Fields = Record<string, string>;
const invalid = (fields: Fields) => Response.json({ code: 'validation', message: 'Invalid input', fields }, { status: 422 });

export function createApp(dbPath = process.env.DB_PATH ?? 'data/tasklet.db') {
  const store = openStore(dbPath);
  const app = new Hono().basePath('/api');

  app.get('/health', (c) => c.json({ ok: true }));

  // F-001
  app.get('/tasks', (c) => c.json({ items: store.list() }));

  // F-002, R-01
  app.post('/tasks', async (c) => {
    const body = (await c.req.json().catch(() => ({}))) as { title?: unknown };
    const title = typeof body.title === 'string' ? body.title.trim() : '';
    if (!title) return invalid({ title: 'Title is required' });
    if (title.length > 120) return invalid({ title: 'Title is too long' });
    return c.json(store.create(title), 201);
  });

  app.onError(() => Response.json({ code: 'internal', message: 'Something went wrong' }, { status: 500 }));

  if (process.env.NODE_ENV === 'test') {
    // Test-only seam: e2e and contract tests reset state between tests. Not part of the public contract.
    app.post('/__test/reset', (c) => {
      store.reset();
      return c.body(null, 204);
    });
  }
  return app;
}
