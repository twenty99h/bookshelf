import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import CommandPalette from "./CommandPalette.svelte";

describe("global command palette", () => {
  afterEach(cleanup);

  it("keeps search failures and the entered query inside the palette", async () => {
    render(CommandPalette, {
      props: {
        open: true,
        query: "репликация",
        results: [],
        onSearch: vi.fn(async () => {
          throw new Error("Индекс временно недоступен");
        }),
        onOpenResult: vi.fn(),
      },
    });

    const input = screen.getByRole("textbox", { name: "Поиск" });
    await fireEvent.click(screen.getByRole("button", { name: "Найти" }));

    expect((await screen.findByRole("alert")).textContent).toContain("Индекс временно недоступен");
    expect((input as HTMLInputElement).value).toBe("репликация");
  });
});
