import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createApp } from './app.ts';

test('health answers ok', async () => {
  const res = await createApp(':memory:').request('/api/health');
  assert.equal(res.status, 200);
  assert.deepEqual(await res.json(), { ok: true });
});

test('F-001 lists tasks newest first', async () => {
  const app = createApp(':memory:');
  await app.request('/api/tasks', { method: 'POST', body: JSON.stringify({ title: 'older' }), headers: { 'content-type': 'application/json' } });
  await app.request('/api/tasks', { method: 'POST', body: JSON.stringify({ title: 'newer' }), headers: { 'content-type': 'application/json' } });
  const res = await app.request('/api/tasks');
  assert.equal(res.status, 200);
  const { items } = (await res.json()) as { items: { title: string }[] };
  assert.deepEqual(items.map((t) => t.title), ['newer', 'older']);
});

test('F-002 rejects blank and over-long titles with field errors', async () => {
  const app = createApp(':memory:');
  for (const [title, msg] of [['  ', 'Title is required'], ['x'.repeat(121), 'Title is too long']]) {
    const res = await app.request('/api/tasks', { method: 'POST', body: JSON.stringify({ title }), headers: { 'content-type': 'application/json' } });
    assert.equal(res.status, 422);
    assert.equal(((await res.json()) as { fields: { title: string } }).fields.title, msg);
  }
});
