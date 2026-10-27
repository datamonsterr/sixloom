import { useCallback, useEffect, useRef, useState } from 'react';
import { api, ApiFailure, type Task } from './api/client';

type Load = { state: 'loading' } | { state: 'error' } | { state: 'ready'; tasks: Task[] };

export default function App() {
  const [load, setLoad] = useState<Load>({ state: 'loading' });
  const [title, setTitle] = useState('');
  const [fieldError, setFieldError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const input = useRef<HTMLInputElement>(null);

  const fetchTasks = useCallback(async () => {
    setLoad({ state: 'loading' });
    try {
      const { items } = await api.listTasks();
      setLoad({ state: 'ready', tasks: items });
      if (items.length === 0) input.current?.focus();
    } catch {
      setLoad({ state: 'error' });
    }
  }, []);

  useEffect(() => {
    void fetchTasks();
  }, [fetchTasks]);

  async function add(e: React.FormEvent) {
    e.preventDefault();
    setFieldError(null);
    if (!title.trim()) {
      setFieldError('Title is required');
      return;
    }
    setSubmitting(true);
    try {
      const t = await api.createTask(title);
      setTitle('');
      setLoad((l) => ({ state: 'ready', tasks: [t, ...(l.state === 'ready' ? l.tasks : [])] }));
    } catch (err) {
      setFieldError(err instanceof ApiFailure ? (err.body.fields?.title ?? err.body.message) : 'Could not add the task');
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <main className="page">
      <h1>Tasklet</h1>
      <form onSubmit={add} className="add" data-sxl="F-002.AC1 F-002.AC2" noValidate>
        <label htmlFor="title" className="sr-only">New task</label>
        <input
          id="title"
          ref={input}
          data-testid="S-01.title"
          placeholder="What needs doing?"
          maxLength={120}
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          aria-invalid={fieldError ? true : undefined}
          aria-describedby={fieldError ? 'title-error' : undefined}
        />
        <button data-testid="S-01.add" disabled={submitting}>Add</button>
        {fieldError && (
          <p id="title-error" role="alert" className="field-error" data-testid="S-01.title-error">
            {fieldError}
          </p>
        )}
      </form>

      {load.state === 'loading' && (
        <ul aria-busy="true" aria-label="Loading tasks" data-testid="S-01.skeleton">
          {[0, 1, 2].map((i) => <li key={i} className="skeleton" />)}
        </ul>
      )}
      {load.state === 'error' && (
        <div role="alert" className="error" data-sxl="F-001.AC3" data-testid="S-01.error">
          <p>Couldn't load tasks.</p>
          <button onClick={() => void fetchTasks()} data-testid="S-01.retry">Retry</button>
        </div>
      )}
      {load.state === 'ready' && load.tasks.length === 0 && (
        <p className="empty" data-sxl="F-001.AC2" data-testid="S-01.empty">All clear. Add your first one above.</p>
      )}
      {load.state === 'ready' && load.tasks.length > 0 && (
        <ul data-sxl="F-001.AC1" data-testid="S-01.list">
          {load.tasks.map((t) => (
            <li key={t.id} data-testid="S-01.row">{t.title}</li>
          ))}
        </ul>
      )}
    </main>
  );
}
