import { test, expect } from './fixtures';

test('F-001.AC1 shows my tasks newest first', async ({ page, open }) => {
  await open('ready');
  await expect(page.getByTestId('S-01.skeleton')).toBeVisible();
  await expect(page.getByTestId('S-01.row')).toHaveText(['Call the landlord', 'Buy coffee beans']);
});

test('F-001.AC2 shows "All clear" when there are none', async ({ page, open }) => {
  await open('empty');
  await expect(page.getByTestId('S-01.empty')).toContainText('All clear');
  await expect(page.getByTestId('S-01.title')).toBeFocused();
});

test('F-001.AC3 shows an error with a Retry that reloads', async ({ page, open }) => {
  await open('error');
  await expect(page.getByTestId('S-01.error')).toContainText("Couldn't load tasks");
  await page.getByTestId('S-01.retry').click();
  await expect(page.getByTestId('S-01.skeleton')).toBeVisible();
});
