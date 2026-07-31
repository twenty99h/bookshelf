import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { LibraryAction, LibraryState } from "@/shared/api";
import { activeLibraryFixture } from "../model/workspace-fixtures";
import WorkspacePage from "./WorkspacePage.svelte";

vi.mock("@lucide/svelte", async () => {
  const { default: Icon } = await import("./IconStub.test.svelte");
  return {
    ArrowLeft: Icon,
    BookOpen: Icon,
    BookCopy: Icon,
    Brain: Icon,
    Check: Icon,
    ChevronDown: Icon,
    ChevronRight: Icon,
    Command: Icon,
    FileArchive: Icon,
    FlaskConical: Icon,
    Gauge: Icon,
    Library: Icon,
    ListTree: Icon,
    Menu: Icon,
    MoreHorizontal: Icon,
    PanelRight: Icon,
    Plus: Icon,
    Search: Icon,
    Settings: Icon,
    Sparkles: Icon,
    StickyNote: Icon,
    X: Icon,
    ZoomIn: Icon,
    ZoomOut: Icon,
  };
});

const execute = vi.fn<(action: LibraryAction) => Promise<LibraryState>>();
const bookUrl = vi.fn(async () => null);
const commandFactory = vi.hoisted(() => vi.fn());
let state: LibraryState;

vi.mock("../api/workspace-commands", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../api/workspace-commands")>()),
  createWorkspaceCommands: commandFactory,
}));

describe("workspace page behavior", () => {
  beforeEach(() => {
    state = activeLibraryFixture();
    commandFactory.mockResolvedValue({
      load: async () => structuredClone(state),
      execute,
      search: async () => [],
      importPdf: async () => null,
      bookUrl,
      exportDraft: async () => null,
      prepareReview: async () => "",
      runReview: async () => structuredClone(state),
      restoreBackup: async () => structuredClone(state),
      exportArchive: async () => true,
      importArchive: async () => structuredClone(state),
      backupMetadata: async () => ({ snapshotAt: 1_785_283_200, archiveAt: null }),
      checkForUpdate: async () => false,
      exportDiagnostics: async () => true,
    });
    execute.mockImplementation(async (action) => {
      if (action.kind === "discardDraft") state.drafts = state.drafts.filter((draft) => draft.id !== action.draftId);
      return structuredClone(state);
    });
  });

  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  it("commits draft deletion through the domain command seam and advances the focus", async () => {
    render(WorkspacePage, { props: { context: "drafts" } });
    await screen.findByText(/conflict resolution happens on the leader/i);

    await fireEvent.click(screen.getByRole("button", { name: "Удалить" }));

    expect(execute).toHaveBeenCalledWith({ kind: "discardDraft", draftId: "draft-001" });
    await waitFor(() => expect(screen.getByText(/selectively simplified/i)).toBeTruthy());
  });

  it("selects a draft from the list and shows every source fragment", async () => {
    render(WorkspacePage, { props: { context: "drafts" } });
    await screen.findByText(/conflict resolution happens on the leader/i);

    await fireEvent.click(screen.getByRole("button", { name: "Все заметки" }));
    const target = screen.getByText(/transactions are an abstraction layer/i).closest("article")!;
    await fireEvent.click(within(target).getByRole("button", { name: "Разобрать" }));

    expect(screen.getByText("Transactions are an abstraction layer")).toBeTruthy();
    expect(screen.getByText("certain concurrency problems do not exist")).toBeTruthy();
    expect((screen.getByRole("button", { name: "Присоединить к выбранной идее" }) as HTMLButtonElement).disabled).toBe(
      true,
    );
  });

  it("selects any listed experiment for editing", async () => {
    render(WorkspacePage, { props: { context: "practice" } });
    await screen.findByRole("heading", { name: "Перепроектирование журнала конфигурации сервиса" });

    await fireEvent.click(screen.getByRole("button", { name: /Сокращение цикла обратной связи при рефакторинге/ }));

    expect(screen.getByRole("heading", { name: "Сокращение цикла обратной связи при рефакторинге" })).toBeTruthy();
    expect((screen.getByRole("textbox", { name: "Наблюдаемый результат" }) as HTMLTextAreaElement).value).toBe(
      "Отрицательный результат выявил слишком крупный первый шаг",
    );
  });

  it("keeps an opening failure local to the recoverable library screen", async () => {
    commandFactory.mockRejectedValueOnce(new Error("Хранилище недоступно"));

    render(WorkspacePage, { props: { context: "dashboard" } });

    expect((await screen.findByRole("alert")).textContent).toContain("Хранилище недоступно");
    expect(screen.getByRole("button", { name: "Повторить открытие" })).toBeTruthy();
  });

  it("persists document mode and image inversion from the reader", async () => {
    render(WorkspacePage, { props: { context: "reader", resourceId: "book-distributed" } });
    await screen.findByText("Designing Data-Intensive Applications");
    await waitFor(() => expect(bookUrl).toHaveBeenCalledWith("book-distributed"));

    await fireEvent.click(screen.getByRole("button", { name: "Тёмный инвертированный режим" }));
    await waitFor(() =>
      expect(execute).toHaveBeenCalledWith(
        expect.objectContaining({
          kind: "updateReaderPreferences",
          bookId: "book-distributed",
          preferences: expect.objectContaining({ documentMode: "darkInverted", invertImages: true }),
        }),
      ),
    );

    await fireEvent.click(screen.getByRole("button", { name: "Не инвертировать изображения" }));
    await waitFor(() =>
      expect(execute).toHaveBeenLastCalledWith(
        expect.objectContaining({
          kind: "updateReaderPreferences",
          preferences: expect.objectContaining({ documentMode: "darkInverted", invertImages: false }),
        }),
      ),
    );
  });

  it("shows persisted reader metadata and retries a failed position save", async () => {
    const book = state.books.find((item) => item.id === "book-distributed")!;
    book.pageCount = 700;
    book.outline = [{ id: "outline", title: "Проверяемая глава", page: 280, parentId: null }];
    execute.mockRejectedValueOnce(new Error("Диск временно недоступен")).mockResolvedValue(structuredClone(state));

    render(WorkspacePage, { props: { context: "reader", resourceId: book.id } });

    await screen.findByText(book.title);
    expect(screen.getByText("Проверяемая глава")).toBeTruthy();
    expect(screen.getByText(/стр\. 286 \/ 700/)).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: "Увеличить масштаб" }));
    expect(await screen.findByText("Не сохранено")).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Повторить сохранение" }));

    await waitFor(() => expect(screen.getByText("Сохранено")).toBeTruthy());
    expect(execute).toHaveBeenCalledTimes(2);
  });

  it("derives dashboard and book progress from persisted state", async () => {
    const book = state.books.find((item) => item.id === "book-distributed")!;
    book.farthestPage = 333;
    state.milestones = [
      { id: "read-old", bookId: book.id, kind: "readingProgress", occurredAt: 1_784_678_400, page: 200 },
      { id: "read-new", bookId: book.id, kind: "readingProgress", occurredAt: 1_785_283_200, page: 333 },
      { id: "idea-new", bookId: book.id, kind: "ideaFormulated", occurredAt: 1_785_283_200, page: null },
      { id: "recall-new", bookId: book.id, kind: "recallCompleted", occurredAt: 1_785_283_200, page: null },
      { id: "experiment-new", bookId: book.id, kind: "experimentAdvanced", occurredAt: 1_785_283_200, page: null },
    ];

    render(WorkspacePage, { props: { context: "dashboard" } });
    const dashboardProgress = (await screen.findByRole("heading", { name: "Текст переходит в знание" })).closest(
      "article",
    );
    expect(dashboardProgress?.textContent).toContain("+133 стр.");
    expect(screen.getByText("333")).toBeTruthy();

    cleanup();
    render(WorkspacePage, { props: { context: "book", resourceId: book.id } });
    await screen.findByRole("heading", { name: book.title, level: 2 });
    expect(screen.getAllByText("333").length).toBeGreaterThan(0);
  });

  it("uses addressable book tabs and keeps destructive actions in the overflow menu", async () => {
    const book = state.books.find((item) => item.id === "book-distributed")!;
    book.outline = [];
    render(WorkspacePage, { props: { context: "book", resourceId: book.id } });
    await screen.findByRole("heading", { name: book.title, level: 2 });

    expect(screen.getByText(/без оглавления · есть текстовый слой/i)).toBeTruthy();
    expect(screen.getByRole("link", { name: "Черновики" }).getAttribute("href")).toContain(`book=${book.id}`);
    expect(screen.queryByRole("button", { name: "Удалить навсегда" })).toBeNull();

    await fireEvent.click(screen.getByRole("button", { name: "Другие действия" }));
    expect(screen.getByRole("button", { name: "Архивировать" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Удалить навсегда" })).toBeTruthy();
  });

  it("derives backup dates and export prompting from persisted activity", async () => {
    render(WorkspacePage, { props: { context: "settings" } });

    await screen.findByRole("heading", { name: "Чтение и рабочее пространство" });
    await fireEvent.click(screen.getByRole("button", { name: "Резервные копии" }));

    expect(screen.queryByText(/30 июля 2026, 18:40/)).toBeNull();
    expect(screen.getByText(/существенные изменения или архив давно не создавался/i)).toBeTruthy();
    await waitFor(() =>
      expect(screen.getByText(/последняя внутренняя копия:/i).textContent).not.toContain("ещё не создавалась"),
    );
  });
});
