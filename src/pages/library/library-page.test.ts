import { cleanup, fireEvent, render, screen, within } from "@testing-library/svelte";
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
  invokeMock.mockResolvedValueOnce({ books: [], workspaceNote: "" }).mockResolvedValueOnce({
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

test("reader can open a local search result with its context", async () => {
  invokeMock.mockImplementation(async (command) => {
    if (command === "search_library") {
      return [
        { id: "topic-1", kind: "topic", title: "Надёжность хранилищ", context: "Распределённые системы · Глава 3" },
      ];
    }
    return {
      books: [{ id: "book-1", title: "Распределённые системы" }],
      ideas: [
        {
          id: "idea-1",
          bookId: "book-1",
          section: "Глава 3",
          formulation: "Кворум ограничивает устаревшие чтения",
          assignments: ["recall"],
          fragments: [],
          versions: [],
          topicIds: ["topic-1"],
        },
      ],
      topics: [{ id: "topic-1", name: "Надёжность хранилищ" }],
      workspaceNote: "",
    };
  });

  render(LibraryPage);
  const search = await screen.findByRole("textbox", { name: "Поиск по книгам, идеям, темам, источникам и материалам" });
  await fireEvent.input(search, { target: { value: "надёжность" } });
  await fireEvent.click(screen.getByRole("button", { name: "Найти" }));
  await fireEvent.click(await screen.findByRole("button", { name: "Открыть Надёжность хранилищ" }));

  const openedRecord = screen.getByRole("region", { name: "Открытая запись поиска" });
  expect(within(openedRecord).getByRole("heading", { name: "Надёжность хранилищ" })).toBeTruthy();
  expect(within(openedRecord).getByText("Распределённые системы · Глава 3")).toBeTruthy();
});

test("Codex receives only the review package after explicit confirmation", async () => {
  const state = {
    books: [{ id: "book-1", title: "Надёжные системы" }],
    ideas: [
      {
        id: "idea-1",
        bookId: "book-1",
        section: "Глава 3",
        formulation: "Отказы нужно проектировать явно",
        assignments: ["recall"],
        fragments: [{ page: 42, excerpt: "Failure is part of the design", context: "secret nearby context" }],
        versions: [],
        topicIds: [],
      },
    ],
    experiments: [
      {
        id: "experiment-1",
        ideaId: "idea-1",
        situation: "SECRET EXPERIMENT",
        action: "",
        result: "",
        conclusion: "",
        successful: false,
        completed: false,
      },
    ],
    workspaceNote: "SECRET NOTE",
  };
  const generatedPackage = [
    "Источник: Надёжные системы, Глава 3, стр. 42",
    "Выбранный фрагмент: Failure is part of the design",
    "Авторская формулировка: Отказы нужно проектировать явно",
  ].join("\n\n");
  invokeMock.mockImplementation(async (command) => (command === "prepare_codex_review" ? generatedPackage : state));
  render(LibraryPage);

  await fireEvent.click(await screen.findByRole("button", { name: /^Идеи/ }));
  await fireEvent.click(screen.getByRole("button", { name: "Проверить идею" }));
  expect(invokeMock).toHaveBeenCalledWith("prepare_codex_review", {
    ideaId: "idea-1",
    requestKind: "ideaReview",
    recallAnswer: undefined,
  });
  expect(screen.getByText(/Полный PDF, эксперименты и другие заметки не добавляются/)).toBeTruthy();
  expect(
    await screen.findByText((_, element) => element?.tagName === "PRE" && element.textContent === generatedPackage),
  ).toBeTruthy();

  await fireEvent.click(screen.getByRole("button", { name: "Подтвердить и отправить" }));
  const reviewCall = invokeMock.mock.calls.find(([command]) => command === "run_codex_review");
  expect(reviewCall).toBeTruthy();
  expect(reviewCall?.[1]).toEqual({
    requestId: expect.any(String),
    ideaId: "idea-1",
    requestKind: "ideaReview",
    recallAnswer: undefined,
    approvedPackage: generatedPackage,
  });
});
