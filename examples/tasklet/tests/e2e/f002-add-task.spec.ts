import { test, expect } from './fixtures';

test('F-002.AC1 adds the task at the top and clears the box', async ({ page, open }) => {
  await open('ready');
  await expect(page.getByTestId('S-01.row').first()).toBeVisible();
  await page.getByTestId('S-01.title').fill('Water the plants');
  await page.getByTestId('S-01.add').click();
  await expect(page.getByTestId('S-01.row').first()).toHaveText('Water the plants');
  await expect(page.getByTestId('S-01.title')).toHaveValue('');
});

test('F-002.AC2 rejects an empty title', async ({ page, open }) => {
  await open('empty');
  await expect(page.getByTestId('S-01.empty')).toBeVisible();
  await page.getByTestId('S-01.add').click();
  await expect(page.getByTestId('S-01.title-error')).toHaveText('Title is required');
  await expect(page.getByTestId('S-01.row')).toHaveCount(0);
});
