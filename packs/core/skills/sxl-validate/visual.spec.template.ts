// sixloom template, verified on a two-feature example app (Tasklet). Keep the structure;
// replace screen IDs, seeds, scenarios and endpoints with your project's.
// Validate loom: the real app must look like the preview, state by state, and pass axe.
// Snapshots are recorded from the preview (E2E_MODE=mock --update-snapshots), then compared in real mode.
import AxeBuilder from '@axe-core/playwright';
import { test, expect, type Scenario } from '../e2e/fixtures';

const states: { name: string; scenario: Scenario; ready: string; act?: (p: import('@playwright/test').Page) => Promise<void> }[] = [
  { name: 'ready', scenario: 'ready', ready: 'S-01.list' },
  { name: 'empty', scenario: 'empty', ready: 'S-01.empty' },
  { name: 'error', scenario: 'error', ready: 'S-01.error' },
  { name: 'validation-error', scenario: 'empty', ready: 'S-01.empty', act: async (p) => { await p.getByTestId('S-01.add').click(); await p.getByTestId('S-01.title-error').waitFor(); } },
];

for (const s of states) {
  test(`S-01 ${s.name} matches the preview and passes axe`, async ({ page, open }) => {
    await open(s.scenario);
    await page.getByTestId(s.ready).waitFor();
    await s.act?.(page);
    await expect(page).toHaveScreenshot(`S-01.${s.name}.png`, { maxDiffPixelRatio: 0.005, animations: 'disabled', caret: 'hide' });
    const { violations } = await new AxeBuilder({ page }).analyze();
    expect(violations.filter((v) => v.impact === 'serious' || v.impact === 'critical').map((v) => v.id)).toEqual([]);
  });
}
