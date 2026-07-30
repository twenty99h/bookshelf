import type { Page } from "@playwright/test";

export class ReaderPage {
  constructor(readonly page: Page) {}

  async open(bookId = "book-distributed") {
    await this.page.goto(`/reader/${bookId}`);
    await this.page.getByTestId("reader-ready").waitFor();
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
