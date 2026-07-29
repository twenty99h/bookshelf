import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { LibraryPage } from ".";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));

const invokeMock = vi.mocked(invoke);
const openMock = vi.mocked(open);

beforeEach(() => {
  invokeMock.mockReset();
  openMock.mockReset();
});

afterEach(cleanup);

test("reader can import the first PDF from the empty library", async () => {
  invokeMock
    .mockResolvedValueOnce({ books: [], workspaceNote: "" })
    .mockResolvedValueOnce({
      books: [{ id: "book-1", title: "Designing Data-Intensive Applications" }],
      workspaceNote: "",
    });
  openMock.mockResolvedValue("/reader/ddia.pdf");

  render(LibraryPage);
  await fireEvent.click(await screen.findByRole("button", { name: "Импортировать PDF" }));

  expect(invokeMock).toHaveBeenLastCalledWith("import_pdf", {
    path: "/reader/ddia.pdf",
    title: "",
  });
  expect(await screen.findByRole("heading", { name: "Designing Data-Intensive Applications" })).toBeTruthy();
});

test("technical workspace note is available as a secondary setting", async () => {
  invokeMock.mockResolvedValueOnce({ books: [], workspaceNote: "" });
  render(LibraryPage);

  await fireEvent.click(await screen.findByRole("button", { name: "Настройки" }));

  expect(screen.getByRole("textbox", { name: "Личное напоминание" })).toBeTruthy();
  expect(screen.getByRole("heading", { name: "Начните с одной важной книги" })).toBeTruthy();
});

test("reader sees an empty library and can save a local workspace note", async () => {
  invokeMock
    .mockResolvedValueOnce({ books: [], workspaceNote: "" })
    .mockResolvedValueOnce({ books: [], workspaceNote: "Выбрать следующую книгу" });

  render(LibraryPage);

  expect(await screen.findByRole("heading", { name: "Начните с одной важной книги" })).toBeTruthy();
  await fireEvent.click(screen.getByRole("button", { name: "Настройки" }));

  const note = screen.getByRole("textbox", { name: "Личное напоминание" });
  await fireEvent.input(note, { target: { value: "Выбрать следующую книгу" } });
  await fireEvent.click(screen.getByRole("button", { name: "Сохранить" }));

  expect((await screen.findByRole("status")).textContent).toContain("Сохранено локально");
  expect(invokeMock).toHaveBeenLastCalledWith("save_workspace_note", {
    note: "Выбрать следующую книгу",
  });
});
