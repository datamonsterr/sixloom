import type { Task } from '../../api/client';

let tasks: Task[] = [
  { id: '6f1c2e1a-2b7d-4a55-9d0e-1f2a3b4c5d6e', title: 'Call the landlord', createdAt: '2026-10-26T09:00:00Z' },
  { id: '0b8e9d3c-4f21-4a7e-8c55-2d6e7f8a9b01', title: 'Buy coffee beans', createdAt: '2026-10-25T18:30:00Z' },
];

export const db = {
  all: () => [...tasks].sort((a, b) => b.createdAt.localeCompare(a.createdAt)),
  create: (title: string): Task => {
    const t = { id: crypto.randomUUID(), title, createdAt: new Date().toISOString() };
    tasks = [t, ...tasks];
    return t;
  },
  clear: () => {
    tasks = [];
  },
};
