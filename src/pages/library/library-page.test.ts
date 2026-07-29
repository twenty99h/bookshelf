import { fireEvent, render, screen } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, expect, test, vi } from "vitest";
import { LibraryPage } from ".";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const invokeMock = vi.mocked(invoke);

beforeEach(() => {
  invokeMock.mockReset();
});

test("reader sees an empty library and can save a local workspace note", async () => {
  invokeMock
    .mockResolvedValueOnce({ books: [], workspaceNote: "" })
    .mockResolvedValueOnce({ books: [], workspaceNote: "Выбрать следующую книгу" });

  render(LibraryPage);

  expect(await screen.findByRole("heading", { name: "Начните с одной важной книги" })).toBeTruthy();

  const note = screen.getByRole("textbox", { name: "Пометка" });
  await fireEvent.input(note, { target: { value: "Выбрать следующую книгу" } });
  await fireEvent.click(screen.getByRole("button", { name: "Сохранить" }));

  expect((await screen.findByRole("status")).textContent).toContain("Сохранено локально");
  expect(invokeMock).toHaveBeenLastCalledWith("save_workspace_note", {
    note: "Выбрать следующую книгу",
  });
});
