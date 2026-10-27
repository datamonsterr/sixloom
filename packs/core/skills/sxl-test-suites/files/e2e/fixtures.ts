// sixloom template, verified on a two-feature example app (Tasklet). Keep the structure;
// replace screen IDs, seeds, scenarios and endpoints with your project's.
import { test as base, expect, type Page } from '@playwright/test';

export type Scenario = 'ready' | 'empty' | 'error';
const real = process.env.E2E_MODE === 'real';
const api = 'http://localhost:8787/api';

// Puts the app in a named state the same way in both modes, then opens it.
async function open(page: Page, scenario: Scenario) {
  if (!real) {
    await page.goto(`/?sxl=${scenario}`);
    return;
  }
  await page.request.post(`${api}/__test/reset`);
  if (scenario === 'ready') {
    await page.request.post(`${api}/tasks`, { data: { title: 'Buy coffee beans' } });
    await page.request.post(`${api}/tasks`, { data: { title: 'Call the landlord' } });
  }
  // Same latency as the mocks, so transient states (skeletons) are observable in both modes.
  await page.route('**/api/tasks', async (r) => {
    if (r.request().method() !== 'GET') return r.continue();
    await new Promise((done) => setTimeout(done, 700));
    if (scenario === 'error') return r.fulfill({ status: 500, json: { code: 'internal', message: 'boom' } });
    return r.continue();
  });
  await page.goto('/');
}

export const test = base.extend<{ open: (s: Scenario) => Promise<void> }>({
  open: async ({ page }, use) => use((s) => open(page, s)),
});
export { expect };
