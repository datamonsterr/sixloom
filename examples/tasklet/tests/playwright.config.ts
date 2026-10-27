import { defineConfig } from '@playwright/test';

// E2E_MODE=mock runs the suite against the preview (MSW); real runs it against apps/api.
const mode = process.env.E2E_MODE ?? 'mock';
const real = mode === 'real';
// Separate ports per mode, so a leftover server from the other mode is never reused.
const webPort = real ? 5174 : 5173;

export default defineConfig({
  timeout: 15_000,
  fullyParallel: !real, // real mode shares one database
  workers: real ? 1 : undefined,
  reporter: process.env.CI ? [['github'], ['html', { open: 'never' }]] : 'list',
  use: { baseURL: `http://localhost:${webPort}`, trace: 'retain-on-failure' },
  projects: [
    { name: 'e2e', testDir: './e2e' },
    { name: 'visual', testDir: './visual' },
    { name: 'contract', testDir: './contract', use: { baseURL: 'http://localhost:8787' } },
  ],
  webServer: [
    // The API always starts: contract tests need it, and real-mode e2e talks to it.
    { command: 'npm --prefix ../apps/api start', url: 'http://localhost:8787/api/health', reuseExistingServer: !process.env.CI, env: { NODE_ENV: 'test', DB_PATH: ':memory:' } },
    {
      command: `npm --prefix ../apps/web run ${real ? 'dev' : 'preview'} -- --port ${webPort} --strictPort`,
      url: `http://localhost:${webPort}`,
      reuseExistingServer: !process.env.CI,
    },
  ],
});
