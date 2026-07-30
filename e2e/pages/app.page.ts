import type { Page } from "@playwright/test";

export class AppPage {
  constructor(readonly page: Page) {}

  async open(path = "/") {
    await this.page.goto(path);
    await this.page.getByText("Локальная библиотека", { exact: true }).waitFor();
  }

  async reset(fixture: "active" | "empty" = "active") {
    await this.page.goto(fixture === "empty" ? "/?fixture=empty" : "/");
    await this.page.evaluate((nextFixture) => window.__BOOKSHELF_TEST__?.reset(nextFixture), fixture);
    await this.page.reload();
  }

  async openCommandPalette() {
    await this.page.keyboard.press("Control+K");
  }
}
