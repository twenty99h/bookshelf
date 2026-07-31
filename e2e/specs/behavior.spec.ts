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
    expect.objectContaining({ kind: "captureDraft", bookId: "book-distributed", page: 10 }),
  );
});

test("reader formulates an idea without assigning learning work automatically", async ({ page }) => {
  const reader = new ReaderPage(page);
  await reader.open();
  await reader.openSidebar("Заметка");
  await page.getByLabel("Фрагмент книги").fill("A timeout cannot prove whether a remote write succeeded.");
  await page.getByRole("button", { name: "Оформить как идею" }).click();
  await page
    .getByLabel("Моя формулировка идеи")
    .fill("Неопределённый результат удалённой записи должен быть явным состоянием модели.");
  await page.getByRole("button", { name: "Создать идею" }).click();

  const commands = await page.evaluate(() => window.__BOOKSHELF_TEST__?.commands ?? []);
  expect(commands).toContainEqual(expect.objectContaining({ kind: "resolveDraftAsIdea", assignments: [] }));
  await new AppPage(page).open("/knowledge");
  await expect(
    page.getByRole("article").getByRole("heading", {
      name: "Неопределённый результат удалённой записи должен быть явным состоянием модели.",
    }),
  ).toBeVisible();
});

test("completed books stay readable but require repeat study before new capture", async ({ page }) => {
  await page.goto("/reader/book-refactoring");
  await expect(page.getByTestId("reader-ready")).toBeVisible();
  await page.getByRole("button", { name: "Показать инструменты чтения" }).click();
  await expect(page.getByText(/Начните повторное изучение/)).toBeVisible();
  await expect(page.getByRole("button", { name: /В черновики/ })).toBeDisabled();
});

test("real PDF text selection preserves an addressable source", async ({ page }) => {
  const reader = new ReaderPage(page);
  await reader.open();
  await expect(page.getByTestId("continuous-pdf")).toBeVisible();
  const textSpan = page.locator(".textLayer span").first();
  await expect(textSpan).toBeVisible();
  await textSpan.evaluate((node) => {
    const range = document.createRange();
    range.selectNodeContents(node);
    const selection = window.getSelection();
    selection?.removeAllRanges();
    selection?.addRange(range);
    document.dispatchEvent(new Event("selectionchange"));
  });
  await expect(page.getByRole("heading", { name: "Черновая заметка" })).toBeVisible();
  await page.getByRole("button", { name: /В черновики/ }).click();

  const commands = await page.evaluate(() => window.__BOOKSHELF_TEST__?.commands ?? []);
  expect(commands).toContainEqual(expect.objectContaining({ kind: "captureDraftSources", bookId: "book-distributed" }));
});

test("cross-page PDF selection saves distinct excerpts and restores both source markers", async ({ page }) => {
  const reader = new ReaderPage(page);
  await reader.open();
  await expect(page.locator("[data-pdf-page]")).toHaveCount(3);
  const renderedPages = page.locator("[data-pdf-page]");
  await expect(renderedPages.nth(0).locator(".textLayer span").first()).toBeVisible();
  await expect(renderedPages.nth(1).locator(".textLayer span").first()).toBeVisible();
  const expected = await page.evaluate(() => {
    const pages = [...document.querySelectorAll<HTMLElement>("[data-pdf-page]")];
    const firstSpans = pages[0]?.querySelectorAll(".textLayer span");
    const secondSpans = pages[1]?.querySelectorAll(".textLayer span");
    const start = firstSpans?.item(Math.max(0, firstSpans.length - 1)).firstChild;
    const end = secondSpans?.item(0).firstChild;
    if (!start || !end) throw new Error("The adjacent PDF text layers were not rendered");
    const range = document.createRange();
    range.setStart(start, 0);
    range.setEnd(end, end.textContent?.length ?? 0);
    const selection = window.getSelection()!;
    selection.removeAllRanges();
    selection.addRange(range);
    document.dispatchEvent(new Event("selectionchange"));
    return [start.textContent?.trim(), end.textContent?.trim()];
  });
  await page.getByRole("button", { name: /В черновики/ }).click();

  const action = await page.evaluate(() =>
    (window.__BOOKSHELF_TEST__?.commands ?? []).findLast(
      (command) => (command as { kind?: string }).kind === "captureDraftSources",
    ),
  );
  expect(action).toMatchObject({
    kind: "captureDraftSources",
    fragments: [expect.objectContaining({ excerpt: expected[0] }), expect.objectContaining({ excerpt: expected[1] })],
  });
  expect((action as { fragments: { excerpt: string }[] }).fragments[0]?.excerpt).not.toBe(
    (action as { fragments: { excerpt: string }[] }).fragments[1]?.excerpt,
  );

  await page.reload();
  const markers = page.getByRole("button", { name: /Открыть сохранённый источник на странице/ });
  await expect(markers).toHaveCount(2);
  await expect(page.locator('[data-source-highlight="true"]').first()).toBeVisible();
  await markers.first().click();
  await expect(page.getByLabel("Фрагмент книги")).toHaveValue(expected[0] ?? "");
});

test("permanent book deletion enumerates consequences and removes the PDF-owned state", async ({ page }) => {
  await new AppPage(page).open("/library/book-domain");
  await page.getByRole("button", { name: "Удалить навсегда" }).click();
  const dialog = page.getByRole("dialog", { name: "Удалить книгу навсегда?" });
  await expect(dialog).toContainText(/сохранённый PDF/i);
  await expect(dialog).toContainText("черновых заметок");
  await dialog.getByRole("button", { name: "Удалить навсегда" }).click();
  await expect(page).toHaveURL(/\/library$/);
  const commands = await page.evaluate(() => window.__BOOKSHELF_TEST__?.commands ?? []);
  expect(commands).toContainEqual({ kind: "deleteBook", bookId: "book-domain" });
});

test("Codex failures stay local and a successful review is resolved by the reader", async ({ page }) => {
  await new AppPage(page).open("/knowledge/idea-leader");
  await page.getByRole("button", { name: "Подготовить проверку" }).click();
  await page.evaluate(() => {
    window.__BOOKSHELF_TEST__!.scenario = "codex-crash";
  });
  await page.getByRole("button", { name: "Запустить проверку" }).click();
  await expect(page.getByText(/Codex завершился с ошибкой/)).toBeVisible();
  await page.evaluate(() => {
    window.__BOOKSHELF_TEST__!.scenario = "success";
  });
  await page.getByRole("button", { name: "Запустить проверку" }).click();
  await expect(page.getByText(/границы применимости/)).toBeVisible();
  await page.getByLabel("Необязательный авторский вывод").fill("Формулировка уже содержит нужное ограничение.");
  await page.getByRole("button", { name: "Оставить без изменений" }).click();

  const commands = await page.evaluate(() => window.__BOOKSHELF_TEST__?.commands ?? []);
  expect(commands).toContainEqual(
    expect.objectContaining({ kind: "resolveReview", ideaId: "idea-leader", decision: "unchanged" }),
  );
});

test("settings sections expose explicit update and diagnostic actions", async ({ page }) => {
  await new AppPage(page).open("/settings");
  await page.getByRole("button", { name: "Библиотека" }).click();
  await page.getByRole("button", { name: "Экспортировать журнал" }).click();
  await expect(page.getByRole("status")).toContainText("экспортирован");
  await page.getByRole("button", { name: "Проверить обновления" }).click();
  await expect(page.getByRole("status").last()).toContainText("актуальная версия");
});

test("browser scenarios expose deterministic loading and recoverable library errors", async ({ page }) => {
  await page.goto("/?scenario=loading");
  await expect(page.getByRole("status")).toContainText("Открываем личную библиотеку");
  await page.goto("/?scenario=error");
  await expect(page.getByRole("alert")).toContainText("Тестовая библиотека недоступна");
});

test("practice supports recall timing and the complete experiment lifecycle", async ({ page }) => {
  await new AppPage(page).open("/practice");
  await page.getByRole("button", { name: "Перенести на 7 дней" }).click();
  await page.getByLabel("Ситуация нового замысла").fill("Проверка модели на новом сервисе");
  await page.getByLabel("Проверяемое действие").fill("Сделать переход владельца явным");
  await page.getByLabel("Следующий шаг нового замысла").fill("Обсудить результат без дедлайна");
  await page.getByRole("button", { name: "Создать замысел" }).click();

  await page.getByRole("button", { name: "Перейти к итогу" }).click();
  await page.getByLabel("Наблюдаемый результат").fill("Команда обнаружила дополнительный сценарий отказа");
  await page.getByLabel("Авторский вывод").fill("Явный переход полезен даже при отрицательном результате");
  await page.getByRole("button", { name: "Завершить эксперимент" }).click();

  const commands = await page.evaluate(() => window.__BOOKSHELF_TEST__?.commands ?? []);
  expect(commands).toContainEqual(expect.objectContaining({ kind: "rescheduleRecall", recallId: "recall-001" }));
  expect(commands).toContainEqual(expect.objectContaining({ kind: "createExperiment" }));
  expect(commands).toContainEqual(expect.objectContaining({ kind: "advanceExperiment", status: "completed" }));
  await expect(page.getByText("Refactoring")).toBeVisible();
});

test("completion persists separate decisions through work, experiments, and final confirmation", async ({ page }) => {
  await new AppPage(page).open("/library/book-distributed/complete");
  await page.getByRole("button", { name: "Чтение завершено" }).click();
  const ideas = page.getByRole("checkbox");
  await ideas.nth(0).check();
  await ideas.nth(1).check();
  await ideas.nth(2).check();
  await page.getByRole("button", { name: "Продолжить" }).click();
  await page.getByLabel("Авторский итог").fill("Теперь я отделяю неопределённость результата от отказа запроса.");
  await page.getByRole("button", { name: "Сохранить черновик и продолжить" }).click();
  const decisions = page.getByLabel("Решение");
  for (let index = 0; index < (await decisions.count()); index += 1) {
    await decisions.nth(index).click();
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Enter");
  }
  await page.getByLabel("Общий комментарий к решениям").fill("Каждый пункт останется в своей очереди.");
  await page.getByRole("button", { name: "Продолжить" }).click();
  await page.getByLabel("Решение").click();
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Enter");
  await page.getByRole("button", { name: "Перейти к подтверждению" }).click();
  await page.waitForFunction(() => {
    const state = JSON.parse(sessionStorage.getItem("bookshelf-browser-fixture") ?? "{}");
    return state.completionDrafts?.[0]?.step === 6;
  });
  await page.reload();
  await expect(page.getByRole("heading", { name: "Подтвердите итог изучения" })).toBeVisible();
  await page.getByRole("button", { name: "Завершить изучение" }).click();

  const persisted = await page.evaluate(() => JSON.parse(sessionStorage.getItem("bookshelf-browser-fixture") ?? "{}"));
  expect(persisted.books.find((book: { id: string }) => book.id === "book-distributed").studyStatus).toBe("completed");
  expect(persisted.experiments[0].status).toBe("running");
  expect(persisted.completionDrafts).toEqual([]);
});

test("activating another book pauses the previous active study", async ({ page }) => {
  await new AppPage(page).open("/library/book-domain");
  await page.getByRole("button", { name: "Сделать активной" }).click();
  await new AppPage(page).open("/");
  await expect(page.getByRole("heading", { name: "Domain-Driven Design" })).toBeVisible();
});

test("draft resolution creates an addressable idea with its source", async ({ page }) => {
  await new AppPage(page).open("/drafts");
  await page.getByLabel("Моя формулировка").fill("Лидерство — это переход доменного риска, а не постоянная роль.");
  await page.getByRole("button", { name: "Создать идею" }).click();
  await new AppPage(page).open("/knowledge");
  await expect(page.getByText("Лидерство — это переход доменного риска, а не постоянная роль.").first()).toBeVisible();
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

test("reader restores document mode, image inversion, and the textless PDF fallback", async ({ page }) => {
  const reader = new ReaderPage(page);
  await reader.open();
  await page.getByRole("button", { name: "Оригинальный режим" }).click();
  await page.getByRole("button", { name: "Тёмный инвертированный режим" }).click();
  await page.getByRole("button", { name: "Не инвертировать изображения" }).click();
  await page.reload();
  await expect(page.getByRole("button", { name: "Тёмный инвертированный режим" })).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  await expect(page.getByRole("button", { name: "Инвертировать изображения" })).toHaveAttribute(
    "aria-pressed",
    "false",
  );

  await page.goto("/reader/book-systems");
  await expect(page.getByTestId("reader-ready")).toBeVisible();
  await page.getByRole("button", { name: "Показать инструменты чтения" }).click();
  await expect(page.getByText(/нет текстового слоя/)).toBeVisible();
  await expect(page.getByText(/OCR не выполняется/)).toBeVisible();
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

test("global command palette opens over Reader and material results return to their PDF source", async ({ page }) => {
  const reader = new ReaderPage(page);
  await reader.open();
  await page.keyboard.press("Control+K");
  const dialog = page.getByRole("dialog", { name: "Быстрый переход" });
  await expect(dialog).toBeVisible();
  await dialog.getByLabel("Поиск").fill("Почему лидер");
  await dialog.getByRole("button", { name: "Найти", exact: true }).click();
  await dialog.getByRole("button", { name: /Почему лидер — это риск/ }).click();

  await expect(page).toHaveURL(/\/reader\/book-distributed\?sourcePage=286$/);
  await expect(page.getByTestId("reader-ready")).toBeVisible();
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
