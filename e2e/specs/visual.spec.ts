import { expect, test } from "@playwright/test";
import { AppPage } from "../pages/app.page";
import { ReaderPage } from "../pages/reader.page";

test.beforeEach(async ({ page }) => {
  await page.addStyleTag({
    content:
      "*,*::before,*::after{animation:none!important;transition:none!important} input,textarea{caret-color:transparent!important}",
  });
});

const screens = [
  ["dashboard-active", "/"],
  ["library-list", "/library"],
  ["book-overview", "/library/book-distributed"],
  ["reader-clean", "/reader/book-distributed"],
  ["drafts-review", "/drafts"],
  ["idea-detail", "/knowledge/idea-leader"],
  ["practice-active", "/practice"],
  ["book-completion", "/library/book-distributed/complete"],
  ["settings-backup", "/settings"],
] as const;

for (const [name, path] of screens) {
  test(name, async ({ page }) => {
    if (name === "reader-clean") {
      await new ReaderPage(page).open();
    } else {
      await new AppPage(page).open(path);
      if (name === "settings-backup") await page.getByRole("button", { name: "Резервные копии" }).click();
    }
    await expect(page).toHaveScreenshot(`${name}.png`);
  });
}

test("reader-note", async ({ page }) => {
  const reader = new ReaderPage(page);
  await reader.open();
  await reader.openSidebar("Заметка");
  await page.getByLabel("Фрагмент книги").fill("conflict resolution happens on the leader");
  await expect(page).toHaveScreenshot("reader-note.png");
});

test("reader-outline", async ({ page }) => {
  const reader = new ReaderPage(page);
  await reader.open();
  await reader.openSidebar("Оглавление");
  await expect(page).toHaveScreenshot("reader-outline.png", { maxDiffPixelRatio: 0.002 });
});

test("first-run", async ({ page }) => {
  await page.goto("/?fixture=empty");
  await page.getByRole("button", { name: "Импортировать PDF" }).waitFor();
  await expect(page).toHaveScreenshot("first-run.png");
});
