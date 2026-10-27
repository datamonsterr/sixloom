import { DatabaseSync } from 'node:sqlite';
import { randomUUID } from 'node:crypto';
import { mkdirSync } from 'node:fs';
import { dirname } from 'node:path';

export type Task = { id: string; title: string; createdAt: string };

export function openStore(path: string) {
  if (path !== ':memory:') mkdirSync(dirname(path), { recursive: true });
  const db = new DatabaseSync(path);
  db.exec(`CREATE TABLE IF NOT EXISTS task (
    id TEXT PRIMARY KEY, title TEXT NOT NULL, created_at TEXT NOT NULL, seq INTEGER NOT NULL)`);
  db.exec('CREATE INDEX IF NOT EXISTS task_order ON task (created_at DESC, seq DESC)');
  let seq = 0;
  return {
    list(): Task[] {
      return db
        .prepare('SELECT id, title, created_at AS createdAt FROM task ORDER BY created_at DESC, seq DESC')
        .all() as Task[];
    },
    create(title: string): Task {
      const t = { id: randomUUID(), title, createdAt: new Date().toISOString() };
      db.prepare('INSERT INTO task VALUES (?, ?, ?, ?)').run(t.id, t.title, t.createdAt, ++seq);
      return t;
    },
    reset() {
      db.exec('DELETE FROM task');
    },
  };
}
