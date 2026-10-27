export type Scenario = 'ready' | 'empty' | 'error' | 'slow' | 'offline';

export function currentScenario(): Scenario {
  const q = new URL(window.location.href).searchParams.get('sxl');
  return (q as Scenario | null) ?? 'ready';
}
