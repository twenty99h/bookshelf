import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { afterEach, expect, test, vi } from "vitest";
import type { LibraryState } from "@/shared/api";
import { LibraryPage, type LibraryCommands } from ".";

afterEach(cleanup);

function libraryState(overrides: Partial<LibraryState> = {}): LibraryState {
  return {
    books: [],
    drafts: [],
    ideas: [],
    topics: [],
    ideaLinks: [],
    experiments: [],
    recalls: [],
    sessions: [],
    materials: [],
    reviews: [],
    workspaceNote: "",
    activeStudyBookId: null,
    weeklySessionBudget: 3,
    lastDebtChange: 0,
    lastDebtChangedAt: 0,
    debtNotificationSentAt: null,
    debtReminderDays: 7,
    ...overrides,
  };
}

function fakeCommands(overrides: Partial<LibraryCommands> = {}): LibraryCommands {
  const initial = libraryState();
  return {
    load: vi.fn(async () => initial),
    execute: vi.fn(async () => initial),
    importPdf: vi.fn(async () => null),
    search: vi.fn(async () => []),
    bookUrl: vi.fn(async () => "asset://book.pdf"),
    exportArchive: vi.fn(async () => false),
    importArchive: vi.fn(async () => null),
    restoreLatestSnapshot: vi.fn(async () => initial),
    exportMaterial: vi.fn(async () => false),
    exportDraft: vi.fn(async () => null),
    installSignedUpdate: vi.fn(async () => false),
    prepareCodexReview: vi.fn(async () => ""),
    runCodexReview: vi.fn(async () => initial),
    cancelCodexReview: vi.fn(async () => undefined),
    startCodexLogin: vi.fn(async () => undefined),
    openExternalUrl: vi.fn(async () => undefined),
    onCodexLogin: vi.fn(async () => () => undefined),
    onCodexReview: vi.fn(async () => () => undefined),
    ...overrides,
  };
}

function book(id: string, title: string): LibraryState["books"][number] {
  return {
    id,
    title,
    storedFile: `books/${id}.pdf`,
    hasTextLayer: true,
    outline: [],
    reading: { page: 1, zoom: 1, scroll: 0 },
    readingCompleted: false,
    studyCompleted: false,
    retrospective: null,
  };
}

function idea(id: string, bookId: string, formulation: string): LibraryState["ideas"][number] {
  return {
    id,
    bookId,
    section: "Глава 1",
    formulation,
    assignments: ["recall"],
    fragments: [{ page: 12, excerpt: "Источник", context: "Контекст" }],
    versions: [],
    topicIds: [],
  };
}

function draft(id: string, bookId: string): LibraryState["drafts"][number] {
  return {
    id,
    bookId,
    section: "Глава 1",
    page: 12,
    excerpt: "Значимый фрагмент",
    context: "Контекст",
    comment: "Моя мысль",
    createdAt: 1_700_000_000,
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((complete) => {
    resolve = complete;
  });
  return { promise, resolve };
}

test("reader can import the first PDF from the empty library", async () => {
  const imported = libraryState({ books: [book("book-1", "Designing Data-Intensive Applications")] });
  const commands = fakeCommands({ importPdf: vi.fn(async () => imported) });

  render(LibraryPage, { props: { commands } });
  await fireEvent.click(await screen.findByRole("button", { name: "Импортировать PDF" }));

  expect(commands.importPdf).toHaveBeenCalledOnce();
  expect(await screen.findByRole("heading", { name: "Designing Data-Intensive Applications" })).toBeTruthy();
});

test("reader sees a loading failure through the page command boundary", async () => {
  const commands = fakeCommands({
    load: vi.fn(async () => Promise.reject({ code: "persistence", message: "Диск недоступен" })),
  });

  render(LibraryPage, { props: { commands } });

  expect((await screen.findByRole("alert")).textContent).toContain("Диск недоступен");
});

test("technical workspace note is available as a secondary setting", async () => {
  const commands = fakeCommands();
  render(LibraryPage, { props: { commands } });

  await fireEvent.click(await screen.findByRole("button", { name: "Настройки" }));

  expect(screen.getByRole("textbox", { name: "Личное напоминание" })).toBeTruthy();
  expect(screen.getByRole("heading", { name: "Начните с одной важной книги" })).toBeTruthy();
});

test("reader can save a local workspace note through the domain action queue", async () => {
  const saved = libraryState({ workspaceNote: "Выбрать следующую книгу" });
  const commands = fakeCommands({ execute: vi.fn(async () => saved) });
  render(LibraryPage, { props: { commands } });

  await fireEvent.click(await screen.findByRole("button", { name: "Настройки" }));
  const note = screen.getByRole("textbox", { name: "Личное напоминание" });
  await fireEvent.input(note, { target: { value: "Выбрать следующую книгу" } });
  await fireEvent.click(screen.getByRole("button", { name: "Сохранить" }));

  expect(
    (await screen.findAllByRole("status")).some((status) => status.textContent?.includes("Сохранено локально")),
  ).toBe(true);
  expect(commands.execute).toHaveBeenCalledWith({
    kind: "saveWorkspaceNote",
    note: "Выбрать следующую книгу",
  });
});

test("reader can open a local search result with its context", async () => {
  const state = libraryState({
    books: [book("book-1", "Распределённые системы")],
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
  });
  const commands = fakeCommands({
    load: vi.fn(async () => state),
    search: vi.fn(async () => [
      {
        id: "topic-1",
        kind: "topic" as const,
        title: "Надёжность хранилищ",
        context: "Распределённые системы · Глава 3",
      },
    ]),
  });

  render(LibraryPage, { props: { commands } });
  const search = await screen.findByRole("textbox", { name: "Поиск по книгам, идеям, темам, источникам и материалам" });
  await fireEvent.input(search, { target: { value: "надёжность" } });
  await fireEvent.click(screen.getByRole("button", { name: "Найти" }));
  await fireEvent.click(await screen.findByRole("button", { name: "Открыть Надёжность хранилищ" }));

  const openedRecord = screen.getByRole("region", { name: "Открытая запись поиска" });
  expect(within(openedRecord).getByRole("heading", { name: "Надёжность хранилищ" })).toBeTruthy();
  expect(within(openedRecord).getByText("Распределённые системы · Глава 3")).toBeTruthy();
});

test("search reports empty results and command errors", async () => {
  const search = vi
    .fn()
    .mockResolvedValueOnce([])
    .mockRejectedValueOnce({ code: "persistence", message: "Индекс недоступен" });
  const commands = fakeCommands({ search });
  render(LibraryPage, { props: { commands } });
  const field = await screen.findByRole("textbox", {
    name: "Поиск по книгам, идеям, темам, источникам и материалам",
  });
  await fireEvent.input(field, { target: { value: "неизвестное" } });
  await fireEvent.click(screen.getByRole("button", { name: "Найти" }));
  expect(await screen.findByText("Совпадений пока нет.")).toBeTruthy();
  await fireEvent.click(screen.getByRole("button", { name: "Найти" }));
  expect((await screen.findByRole("alert")).textContent).toContain("Индекс недоступен");
});

test("reader exports a selected authored material", async () => {
  const state = libraryState({
    books: [book("book-1", "Надёжные системы")],
    ideas: [idea("idea-1", "book-1", "Отказы проектируются явно")],
    materials: [
      {
        id: "material-1",
        title: "Учения по отказу",
        problem: "Сбой",
        idea: "Тренировать переключение",
        example: "Реплика",
        result: "Доступность",
        limitations: "Стоимость",
        ideaIds: ["idea-1"],
      },
    ],
  });
  const commands = fakeCommands({ load: vi.fn(async () => state), exportMaterial: vi.fn(async () => true) });
  render(LibraryPage, { props: { commands } });
  await fireEvent.click(await screen.findByRole("button", { name: /^Идеи/ }));
  await fireEvent.click(await screen.findByRole("button", { name: "Сохранить Markdown" }));
  expect(commands.exportMaterial).toHaveBeenCalledWith("material-1", "Учения по отказу");
});

test("archive export, import, and snapshot restore return through the settings boundary", async () => {
  const restored = libraryState({ workspaceNote: "Восстановлено" });
  const commands = fakeCommands({
    exportArchive: vi.fn(async () => true),
    importArchive: vi.fn(async () => restored),
    restoreLatestSnapshot: vi.fn(async () => restored),
  });
  render(LibraryPage, { props: { commands } });
  await fireEvent.click(await screen.findByRole("button", { name: "Настройки" }));
  await fireEvent.input(screen.getByLabelText("Пароль архива"), { target: { value: "надёжный пароль" } });
  await fireEvent.click(screen.getByRole("button", { name: "Экспортировать" }));
  await waitFor(() => expect(commands.exportArchive).toHaveBeenCalledWith("надёжный пароль"));
  await fireEvent.click(screen.getByRole("button", { name: "Импортировать" }));
  await waitFor(() => expect(commands.importArchive).toHaveBeenCalledWith("надёжный пароль"));
  await fireEvent.click(screen.getByRole("button", { name: "Восстановить снимок" }));
  expect(commands.restoreLatestSnapshot).toHaveBeenCalledOnce();
});

test("draft lifecycle uses only the injected domain command seam", async () => {
  const state = libraryState({
    books: [book("book-1", "Надёжные системы")],
    drafts: [draft("draft-1", "book-1")],
    ideas: [idea("idea-1", "book-1", "Отказы проектируются явно")],
  });
  const commands = fakeCommands({
    load: vi.fn(async () => state),
    execute: vi.fn(async () => state),
    exportDraft: vi.fn(async () => state),
  });
  render(LibraryPage, { props: { commands } });

  await fireEvent.click(await screen.findByRole("button", { name: /^Очередь разбора/ }));
  await fireEvent.input(screen.getByRole("textbox", { name: "Самостоятельная формулировка" }), {
    target: { value: "Система должна ожидать отказ" },
  });
  await fireEvent.click(screen.getByRole("button", { name: "Создать идею" }));
  await waitFor(() =>
    expect(commands.execute).toHaveBeenCalledWith({
      kind: "resolveDraftAsIdea",
      draftId: "draft-1",
      formulation: "Система должна ожидать отказ",
      section: "Введение",
      assignments: ["recall"],
    }),
  );

  const ideaSelect = screen.getByRole("button", { name: "Идея для присоединения" });
  await fireEvent.keyDown(ideaSelect, { key: "ArrowDown" });
  await fireEvent.keyDown(ideaSelect, { key: "Enter" });
  await fireEvent.click(screen.getByRole("button", { name: "Присоединить" }));
  await waitFor(() =>
    expect(commands.execute).toHaveBeenCalledWith({
      kind: "attachDraftToIdea",
      draftId: "draft-1",
      ideaId: "idea-1",
    }),
  );

  await fireEvent.click(screen.getByRole("button", { name: "Экспортировать" }));
  expect(commands.exportDraft).toHaveBeenCalledWith("draft-1");
  await fireEvent.click(screen.getByRole("button", { name: "Удалить" }));
  await waitFor(() => expect(commands.execute).toHaveBeenCalledWith({ kind: "discardDraft", draftId: "draft-1" }));
});

test("reader opens a PDF and saves its source and restored position", async () => {
  const openedBook = {
    ...book("book-1", "Надёжные системы"),
    reading: { page: 12, zoom: 1.4, scroll: 320 },
  };
  const state = libraryState({ books: [openedBook] });
  const commands = fakeCommands({ load: vi.fn(async () => state), execute: vi.fn(async () => state) });
  render(LibraryPage, { props: { commands } });

  await fireEvent.click(await screen.findByRole("button", { name: "Продолжить" }));
  expect(commands.bookUrl).toHaveBeenCalledWith("book-1");
  expect(await screen.findByRole("region", { name: "Встроенный PDF.js просмотрщик" })).toBeTruthy();

  await fireEvent.click(screen.getByRole("button", { name: "Сохранить сейчас" }));
  await waitFor(() =>
    expect(commands.execute).toHaveBeenCalledWith({
      kind: "updateReading",
      bookId: "book-1",
      page: 12,
      zoom: 1.4,
      scroll: 320,
    }),
  );

  await fireEvent.input(screen.getByRole("textbox", { name: "Выделенный фрагмент" }), {
    target: { value: "Failure is part of the design" },
  });
  await fireEvent.input(screen.getByRole("textbox", { name: "Непосредственный контекст" }), {
    target: { value: "Граница отказа" },
  });
  await fireEvent.click(screen.getByRole("button", { name: "В очередь разбора" }));
  await waitFor(() =>
    expect(commands.execute).toHaveBeenCalledWith({
      kind: "captureDraft",
      bookId: "book-1",
      section: "Введение",
      page: 12,
      excerpt: "Failure is part of the design",
      context: "Граница отказа",
      comment: "",
    }),
  );
});

test("reader refines, assigns, and explicitly links book ideas", async () => {
  const state = libraryState({
    books: [book("book-1", "Надёжные системы")],
    ideas: [
      idea("idea-1", "book-1", "Отказы проектируются явно"),
      idea("idea-2", "book-1", "Резервирование ограничивает простой"),
    ],
  });
  const commands = fakeCommands({ load: vi.fn(async () => state), execute: vi.fn(async () => state) });
  render(LibraryPage, { props: { commands } });

  await fireEvent.click(await screen.findByRole("button", { name: /^Идеи/ }));
  const develop = await screen.findAllByRole("button", { name: "Развить идею" });
  await fireEvent.click(develop[0]!);
  const formulation = screen.getByRole("textbox", { name: "Текущая формулировка" });
  await fireEvent.input(formulation, { target: { value: "Отказ — ожидаемое состояние системы" } });
  await fireEvent.click(screen.getByRole("checkbox", { name: "Передача" }));
  await fireEvent.click(screen.getByRole("button", { name: "Сохранить новую версию" }));
  await waitFor(() =>
    expect(commands.execute).toHaveBeenCalledWith({
      kind: "updateIdea",
      ideaId: "idea-1",
      formulation: "Отказ — ожидаемое состояние системы",
      assignments: ["recall", "transfer"],
    }),
  );

  const linkedIdea = screen.getByRole("button", { name: "Связанная идея" });
  await fireEvent.keyDown(linkedIdea, { key: "ArrowDown" });
  await fireEvent.keyDown(linkedIdea, { key: "Enter" });
  await fireEvent.click(screen.getByRole("button", { name: "Подтвердить связь" }));
  await waitFor(() =>
    expect(commands.execute).toHaveBeenCalledWith({
      kind: "linkIdeas",
      fromIdeaId: "idea-1",
      toIdeaId: "idea-2",
      relation: "complements",
    }),
  );
});

test.each([
  ["Уверенно", "confident"],
  ["Частично", "partial"],
  ["Не восстановил", "notRecalled"],
] as const)("reader records the canonical recall result %s", async (buttonName, rating) => {
  const state = libraryState({
    books: [book("book-1", "Надёжные системы")],
    ideas: [idea("idea-1", "book-1", "Отказы проектируются явно")],
  });
  const commands = fakeCommands({ load: vi.fn(async () => state), execute: vi.fn(async () => state) });
  render(LibraryPage, { props: { commands } });

  await fireEvent.click(await screen.findByRole("button", { name: /^Идеи/ }));
  await fireEvent.click(await screen.findByRole("button", { name: "Восстановить знание" }));
  await fireEvent.input(screen.getByRole("textbox", { name: "Мой ответ" }), {
    target: { value: "Отказ нужно считать штатным состоянием" },
  });
  await fireEvent.click(screen.getByRole("button", { name: "Свериться с идеей" }));
  await fireEvent.click(screen.getByRole("button", { name: buttonName }));

  await waitFor(() =>
    expect(commands.execute).toHaveBeenCalledWith({
      kind: "completeRecall",
      ideaId: "idea-1",
      answer: "Отказ нужно считать штатным состоянием",
      rating,
      nextAt: null,
    }),
  );
});

test("recall validation error stays visible without losing the form", async () => {
  const state = libraryState({
    books: [book("book-1", "Надёжные системы")],
    ideas: [idea("idea-1", "book-1", "Отказы проектируются явно")],
  });
  const commands = fakeCommands({
    load: vi.fn(async () => state),
    execute: vi.fn(async () => Promise.reject({ code: "validation", message: "Ответ не принят" })),
  });
  render(LibraryPage, { props: { commands } });

  await fireEvent.click(await screen.findByRole("button", { name: /^Идеи/ }));
  await fireEvent.click(await screen.findByRole("button", { name: "Восстановить знание" }));
  await fireEvent.input(screen.getByRole("textbox", { name: "Мой ответ" }), { target: { value: "Мой ответ" } });
  await fireEvent.click(screen.getByRole("button", { name: "Свериться с идеей" }));
  await fireEvent.click(screen.getByRole("button", { name: "Частично" }));

  expect((await screen.findByRole("alert")).textContent).toContain("Ответ не принят");
});

test("study planning and practical experiment actions stay on the command seam", async () => {
  const state = libraryState({
    books: [book("book-1", "Надёжные системы")],
    ideas: [idea("idea-1", "book-1", "Отказы проектируются явно")],
    sessions: [
      {
        id: "session-1",
        intention: "Разобрать заметки",
        plannedAt: 1_700_000_000,
        status: "planned",
        resolutionReason: "",
        debtAtStart: 0,
      },
    ],
  });
  const commands = fakeCommands({ load: vi.fn(async () => state), execute: vi.fn(async () => state) });
  render(LibraryPage, { props: { commands } });

  await fireEvent.click(await screen.findByRole("button", { name: "Изучение" }));
  await fireEvent.input(screen.getByRole("textbox", { name: "Намерение сеанса" }), {
    target: { value: "Продолжить главу" },
  });
  await fireEvent.click(screen.getByRole("button", { name: "Запланировать" }));
  await waitFor(() => expect(commands.execute).toHaveBeenCalledWith(expect.objectContaining({ kind: "planSession" })));
  await fireEvent.click(screen.getByRole("button", { name: "Начать сеанс" }));
  await waitFor(() => expect(commands.execute).toHaveBeenCalledWith({ kind: "startSession", sessionId: "session-1" }));

  await fireEvent.click(screen.getByRole("button", { name: /^Идеи/ }));
  await fireEvent.click(await screen.findByRole("button", { name: "Развить идею" }));
  for (const [name, value] of [
    ["Ситуация", "Сбой узла"],
    ["Действие", "Переключить реплику"],
    ["Наблюдаемый результат", "Сервис доступен"],
    ["Мой вывод", "Переключение нужно репетировать"],
  ] as const) {
    await fireEvent.input(screen.getByRole("textbox", { name }), { target: { value } });
  }
  await fireEvent.click(screen.getByRole("checkbox", { name: "Результат оказался положительным" }));
  await fireEvent.click(screen.getByRole("button", { name: "Завершить эксперимент" }));
  await waitFor(() =>
    expect(commands.execute).toHaveBeenCalledWith({
      kind: "completeExperiment",
      ideaId: "idea-1",
      situation: "Сбой узла",
      action: "Переключить реплику",
      result: "Сервис доступен",
      conclusion: "Переключение нужно репетировать",
      successful: true,
    }),
  );
});

test("study completion requires an authored retrospective and selected ideas", async () => {
  const state = libraryState({
    books: [book("book-1", "Надёжные системы")],
    ideas: [
      idea("idea-1", "book-1", "Идея один"),
      idea("idea-2", "book-1", "Идея два"),
      idea("idea-3", "book-1", "Идея три"),
    ],
    activeStudyBookId: "book-1",
  });
  const commands = fakeCommands({ load: vi.fn(async () => state), execute: vi.fn(async () => state) });
  render(LibraryPage, { props: { commands } });
  await fireEvent.click(await screen.findByRole("button", { name: "Изучение" }));

  await fireEvent.input(
    screen.getByRole("textbox", { name: "Результаты применения и изменения в понимании или действиях" }),
    { target: { value: "Начал проектировать отказоустойчивость заранее" } },
  );
  for (const name of ["Идея один", "Идея два", "Идея три"]) {
    await fireEvent.click(screen.getByRole("checkbox", { name }));
  }
  await fireEvent.input(screen.getByRole("textbox", { name: "Продолжающиеся эксперименты или восстановления" }), {
    target: { value: "Продолжить учения" },
  });
  await fireEvent.input(screen.getByRole("textbox", { name: "Решение по оставшемуся долгу" }), {
    target: { value: "Разобрать позже" },
  });
  await fireEvent.click(screen.getByRole("button", { name: "Завершить изучение" }));

  await waitFor(() =>
    expect(commands.execute).toHaveBeenCalledWith({
      kind: "completeStudy",
      bookId: "book-1",
      retrospective: "Начал проектировать отказоустойчивость заранее",
      significantIdeaIds: ["idea-1", "idea-2", "idea-3"],
      continuingWork: "Продолжить учения",
      debtDecision: "Разобрать позже",
    }),
  );
});

test("mutating actions execute in order and replace the snapshot atomically", async () => {
  const first = deferred<LibraryState>();
  const second = deferred<LibraryState>();
  const execute = vi.fn().mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise);
  const commands = fakeCommands({ execute });
  render(LibraryPage, { props: { commands } });
  await fireEvent.click(await screen.findByRole("button", { name: "Изучение" }));

  await fireEvent.click(screen.getByRole("button", { name: "2" }));
  await fireEvent.click(screen.getByRole("button", { name: "3" }));
  expect(execute).toHaveBeenCalledTimes(1);

  first.resolve(libraryState({ weeklySessionBudget: 2 }));
  await screen.findByRole("heading", { name: "2 сеанса" });
  expect(execute).toHaveBeenCalledTimes(2);
  second.resolve(libraryState({ weeklySessionBudget: 3 }));
  expect(await screen.findByRole("heading", { name: "3 сеанса" })).toBeTruthy();
});

test("snapshot-producing mutations execute in request order", async () => {
  const slowImport = deferred<LibraryState | null>();
  const execute = vi.fn(async () => libraryState({ weeklySessionBudget: 2 }));
  const commands = fakeCommands({
    importPdf: vi.fn(() => slowImport.promise),
    execute,
  });
  render(LibraryPage, { props: { commands } });
  await fireEvent.click(await screen.findByRole("button", { name: "Изучение" }));

  void fireEvent.click(screen.getByRole("button", { name: "Импортировать PDF" }));
  await fireEvent.click(screen.getByRole("button", { name: "2" }));
  expect(execute).not.toHaveBeenCalled();
  slowImport.resolve(libraryState({ weeklySessionBudget: 5, books: [book("late", "Устаревшая книга")] }));

  await vi.waitFor(() => expect(execute).toHaveBeenCalledOnce());
  await fireEvent.click(screen.getByRole("button", { name: "Изучение" }));
  expect(await screen.findByRole("heading", { name: "2 сеанса" })).toBeTruthy();
  expect(screen.queryByText("Устаревшая книга")).toBeNull();
});

test("snapshot restore is serialized with a later mutation", async () => {
  const slowRestore = deferred<LibraryState>();
  const execute = vi.fn(async () => libraryState({ weeklySessionBudget: 4 }));
  const commands = fakeCommands({
    restoreLatestSnapshot: vi.fn(() => slowRestore.promise),
    execute,
  });
  render(LibraryPage, { props: { commands } });
  await fireEvent.click(await screen.findByRole("button", { name: "Настройки" }));
  void fireEvent.click(screen.getByRole("button", { name: "Восстановить снимок" }));
  await fireEvent.click(screen.getByRole("button", { name: "Изучение" }));
  await fireEvent.click(screen.getByRole("button", { name: "4" }));
  expect(execute).not.toHaveBeenCalled();

  slowRestore.resolve(libraryState({ weeklySessionBudget: 2, workspaceNote: "Устаревший снимок" }));
  await vi.waitFor(() => expect(execute).toHaveBeenCalledOnce());
  expect(await screen.findByRole("heading", { name: "4 сеанса" })).toBeTruthy();
});

test("Codex receives only the review package after explicit confirmation", async () => {
  const state = libraryState({
    books: [book("book-1", "Надёжные системы")],
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
  });
  const generatedPackage = [
    "Источник: Надёжные системы, Глава 3, стр. 42",
    "Выбранный фрагмент: Failure is part of the design",
    "Авторская формулировка: Отказы нужно проектировать явно",
  ].join("\n\n");
  const commands = fakeCommands({
    load: vi.fn(async () => state),
    prepareCodexReview: vi.fn(async () => generatedPackage),
    runCodexReview: vi.fn(async () => state),
  });
  render(LibraryPage, { props: { commands } });

  await fireEvent.click(await screen.findByRole("button", { name: /^Идеи/ }));
  await fireEvent.click(screen.getByRole("button", { name: "Проверить идею" }));
  expect(commands.prepareCodexReview).toHaveBeenCalledWith("idea-1", "ideaReview", undefined);
  expect(screen.getByText(/Полный PDF, эксперименты и другие заметки не добавляются/)).toBeTruthy();
  expect(
    await screen.findByText((_, element) => element?.tagName === "PRE" && element.textContent === generatedPackage),
  ).toBeTruthy();

  await fireEvent.click(screen.getByRole("button", { name: "Подтвердить и отправить" }));
  expect(commands.runCodexReview).toHaveBeenCalledWith(
    expect.any(String),
    "idea-1",
    "ideaReview",
    generatedPackage,
    undefined,
  );
});

test("reader can cancel a running Codex review", async () => {
  const state = libraryState({
    books: [book("book-1", "Надёжные системы")],
    ideas: [idea("idea-1", "book-1", "Отказы проектируются явно")],
  });
  const running = deferred<LibraryState>();
  const commands = fakeCommands({
    load: vi.fn(async () => state),
    prepareCodexReview: vi.fn(async () => "Проверяемый пакет"),
    runCodexReview: vi.fn(() => running.promise),
  });
  render(LibraryPage, { props: { commands } });
  await fireEvent.click(await screen.findByRole("button", { name: /^Идеи/ }));
  await fireEvent.click(screen.getByRole("button", { name: "Проверить идею" }));
  await fireEvent.click(await screen.findByRole("button", { name: "Подтвердить и отправить" }));
  await fireEvent.click(await screen.findByRole("button", { name: "Отменить" }));
  expect(commands.cancelCodexReview).toHaveBeenCalledWith(expect.any(String));
  running.resolve(state);
});

test("Codex process failure remains an actionable page error", async () => {
  const state = libraryState({
    books: [book("book-1", "Надёжные системы")],
    ideas: [idea("idea-1", "book-1", "Отказы проектируются явно")],
  });
  const commands = fakeCommands({
    load: vi.fn(async () => state),
    prepareCodexReview: vi.fn(async () => "Проверяемый пакет"),
    runCodexReview: vi.fn(async () => Promise.reject({ code: "external_process", message: "Codex завершился" })),
  });
  render(LibraryPage, { props: { commands } });
  await fireEvent.click(await screen.findByRole("button", { name: /^Идеи/ }));
  await fireEvent.click(screen.getByRole("button", { name: "Проверить идею" }));
  await fireEvent.click(await screen.findByRole("button", { name: "Подтвердить и отправить" }));
  expect((await screen.findByRole("alert")).textContent).toContain("Codex завершился");
});
