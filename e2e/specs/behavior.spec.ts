import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";
import { AppPage } from "../pages/app.page";
import { ReaderPage } from "../pages/reader.page";

test("reader saves a draft through the domain command seam", async ({ page }) => {
  const reader = new ReaderPage(page);
  await reader.open();
  await reader.openSidebar("Заметка");
  await reader.saveFixtureDraft("Проверить границу failover");

  const commands = await page.evaluate(() => window.__BOOKSHELF_TEST__?.commands ?? []);
  expect(commands).toContainEqual(
    expect.objectContaining({ kind: "captureDraft", bookId: "book-distributed", page: 286 }),
  );
});

test("reader restores position and layout after reload", async ({ page }) => {
  const reader = new ReaderPage(page);
  await reader.open();
  await reader.openSidebar("Оглавление");
  await page.getByRole("button", { name: /Введение/ }).click();
  await page.reload();
  await expect(page.getByText("стр. 1 / 612")).toBeVisible();
  await expect(page.getByRole("button", { name: "Показать инструменты чтения" })).toHaveAttribute(
    "aria-expanded",
    "true",
  );
  await expect(page.getByRole("heading", { name: "Оглавление" })).toBeVisible();
});

test("keyboard-only reader flow returns focus to the sidebar trigger", async ({ page }) => {
  const reader = new ReaderPage(page);
  await reader.open();
  await page.getByRole("button", { name: "Показать инструменты чтения" }).focus();
  await page.keyboard.press("Enter");
  await page.getByRole("button", { name: "Поиск", exact: true }).click();
  await page.getByLabel("Текстовый слой").fill("leader");
  await page.keyboard.press("Escape");
  await expect(page.getByRole("button", { name: "Показать инструменты чтения" })).toBeFocused();
});

test("global command palette navigates in the current window", async ({ page }) => {
  const app = new AppPage(page);
  await app.open();
  await app.openCommandPalette();
  await page.getByLabel("Поиск").fill("Domain-Driven");
  await page.getByRole("button", { name: "Найти", exact: true }).click();
  await page.getByRole("button", { name: /Domain-Driven Design/ }).click();
  await expect(page).toHaveURL(/\/library\/book-domain$/);
});

for (const path of ["/", "/library", "/drafts", "/knowledge", "/practice", "/settings"]) {
  test(`has no automatically detectable accessibility violations at ${path}`, async ({ page }) => {
    await new AppPage(page).open(path);
    const results = await new AxeBuilder({ page }).analyze();
    expect(results.violations).toEqual([]);
  });
}

test.describe("compact desktop boundary", () => {
  test.use({ viewport: { width: 1280, height: 720 } });

  test("reader sidebar becomes an overlay without horizontal page scrolling", async ({ page }) => {
    const reader = new ReaderPage(page);
    await reader.open();
    await reader.openSidebar("Оглавление");
    const pageWidth = await page.evaluate(() => document.documentElement.scrollWidth);
    expect(pageWidth).toBe(1280);
  });
});
