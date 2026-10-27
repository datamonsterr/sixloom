import { test, expect } from '@playwright/test';
import { conforms } from './contract';

test.beforeEach(async ({ request }) => {
  await request.post('/api/__test/reset');
});

test('listTasks 200 returns tasks newest first (F-001.AC1)', async ({ request }) => {
  await request.post('/api/tasks', { data: { title: 'older' } });
  await request.post('/api/tasks', { data: { title: 'newer' } });
  const res = await request.get('/api/tasks');
  expect(res.status()).toBe(200);
  const body = await res.json();
  expect(conforms('/tasks', 'get', 200, body)).toBeNull();
  expect(body.items.map((t: { title: string }) => t.title)).toEqual(['newer', 'older']);
});

test('createTask 201 returns the trimmed task (F-002.AC1)', async ({ request }) => {
  const res = await request.post('/api/tasks', { data: { title: '  Water the plants  ' } });
  expect(res.status()).toBe(201);
  const body = await res.json();
  expect(conforms('/tasks', 'post', 201, body)).toBeNull();
  expect(body.title).toBe('Water the plants');
});

test('createTask 422 on an empty title (F-002.AC2)', async ({ request }) => {
  const res = await request.post('/api/tasks', { data: { title: '   ' } });
  expect(res.status()).toBe(422);
  const body = await res.json();
  expect(conforms('/tasks', 'post', 422, body)).toBeNull();
  expect(body.fields.title).toBe('Title is required');
});

test('createTask 422 on a title over 120 characters (R-01)', async ({ request }) => {
  const res = await request.post('/api/tasks', { data: { title: 'x'.repeat(121) } });
  expect(res.status()).toBe(422);
  expect(conforms('/tasks', 'post', 422, await res.json())).toBeNull();
});
