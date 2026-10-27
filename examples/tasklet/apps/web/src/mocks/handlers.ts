import { http, HttpResponse, delay } from 'msw';
import { currentScenario } from './scenarios';
import { db } from './fixtures/db';

const latency = { list: 700, write: 300 };

export const handlers = [
  http.get('/api/tasks', async () => {
    const s = currentScenario();
    await delay(s === 'slow' ? 3000 : latency.list);
    if (s === 'offline') return HttpResponse.error();
    if (s === 'error') return HttpResponse.json({ code: 'internal', message: 'Something went wrong' }, { status: 500 });
    return HttpResponse.json({ items: s === 'empty' ? [] : db.all() });
  }),
  http.post('/api/tasks', async ({ request }) => {
    await delay(latency.write);
    const { title } = (await request.json()) as { title?: string };
    const t = title?.trim() ?? '';
    if (!t) return HttpResponse.json({ code: 'validation', message: 'Invalid input', fields: { title: 'Title is required' } }, { status: 422 });
    if (t.length > 120) return HttpResponse.json({ code: 'validation', message: 'Invalid input', fields: { title: 'Title is too long' } }, { status: 422 });
    if (currentScenario() === 'empty') db.clear();
    return HttpResponse.json(db.create(t), { status: 201 });
  }),
];
