// The only module that talks to the network. VITE_API_MODE=mock serves it from MSW.
import type { components } from './schema';

export type Task = components['schemas']['Task'];
// `offline` never comes from the server; the client raises it when fetch itself fails.
export type ApiError = components['schemas']['Error'] | { code: 'offline'; message: string; fields?: undefined };

export class ApiFailure extends Error {
  status: number;
  body: ApiError;
  constructor(status: number, body: ApiError) {
    super(body.message);
    this.status = status;
    this.body = body;
  }
}

const base = import.meta.env.VITE_API_URL ?? '/api';

async function call<T>(path: string, init?: RequestInit): Promise<T> {
  let res: Response;
  try {
    res = await fetch(base + path, { headers: { 'content-type': 'application/json' }, ...init });
  } catch {
    throw new ApiFailure(0, { code: 'offline', message: 'Network unavailable' });
  }
  const body = res.status === 204 ? undefined : await res.json();
  if (!res.ok) throw new ApiFailure(res.status, body as ApiError);
  return body as T;
}

export const api = {
  listTasks: () => call<{ items: Task[] }>('/tasks'),
  createTask: (title: string) => call<Task>('/tasks', { method: 'POST', body: JSON.stringify({ title }) }),
};
