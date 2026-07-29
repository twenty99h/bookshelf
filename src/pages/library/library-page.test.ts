import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { LibraryPage } from ".";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn().mockResolvedValue(() => {}) }));

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

test("Codex receives only the review package after explicit confirmation", async () => {
  const state = {
    books: [{ id: "book-1", title: "Надёжные системы" }],
    ideas: [{
      id: "idea-1", bookId: "book-1", section: "Глава 3", formulation: "Отказы нужно проектировать явно",
      assignments: ["recall"], fragments: [{ page: 42, excerpt: "Failure is part of the design", context: "secret nearby context" }],
      versions: [], topicIds: [],
    }],
    experiments: [{ id: "experiment-1", ideaId: "idea-1", situation: "SECRET EXPERIMENT", action: "", result: "", conclusion: "", successful: false, completed: false }],
    workspaceNote: "SECRET NOTE",
  };
  invokeMock.mockResolvedValue(state);
  render(LibraryPage);

  await fireEvent.click(await screen.findByRole("button", { name: /^Идеи/ }));
  await fireEvent.click(screen.getByRole("button", { name: "Проверить идею" }));
  expect(invokeMock).toHaveBeenCalledTimes(1);
  expect(screen.getByText(/Полный PDF, эксперименты и другие заметки не добавляются/)).toBeTruthy();

  await fireEvent.click(screen.getByRole("button", { name: "Подтвердить и отправить" }));
  const reviewCall = invokeMock.mock.calls.find(([command]) => command === "run_codex_review");
  expect(reviewCall).toBeTruthy();
  const payload = reviewCall?.[1] as { package: string };
  expect(payload.package).toContain("Отказы нужно проектировать явно");
  expect(payload.package).toContain("Failure is part of the design");
  expect(payload.package).not.toContain("SECRET NOTE");
  expect(payload.package).not.toContain("SECRET EXPERIMENT");
  expect(payload.package).not.toContain("secret nearby context");
});
