import type { Page } from "@playwright/test";

export class ReaderPage {
  constructor(readonly page: Page) {}

  async open(bookId = "book-distributed") {
    await this.page.goto(`/reader/${bookId}`);
    await this.page.getByTestId("reader-ready").waitFor();
    await this.page.locator('[data-pdf-page][aria-busy="false"]').first().waitFor();
    await this.page.waitForFunction(() => document.querySelectorAll('[data-pdf-page][aria-busy="true"]').length === 0);
  }

  async openSidebar(tab: "Заметка" | "Оглавление" | "Поиск") {
    await this.page.getByRole("button", { name: "Показать инструменты чтения" }).click();
    await this.page.getByRole("button", { name: tab }).click();
  }

  async saveFixtureDraft(comment: string) {
    await this.page.getByLabel("Фрагмент книги").fill("conflict resolution happens on the leader");
    await this.page.getByLabel("Моя мысль (необязательно)").fill(comment);
    await this.page.keyboard.press("Control+Enter");
  }
}
