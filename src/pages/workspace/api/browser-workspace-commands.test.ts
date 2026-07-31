import { beforeEach, describe, expect, it } from "vitest";
import { browserWorkspaceCommands } from "./browser-workspace-commands";

describe("browser workspace command seam", () => {
  beforeEach(() => window.__BOOKSHELF_TEST__?.reset("active"));

  it("atomically pauses the prior active study", async () => {
    const state = await browserWorkspaceCommands.execute({ kind: "activateStudy", bookId: "book-domain" });

    expect(state.activeStudyBookId).toBe("book-domain");
    expect(state.books.find((book) => book.id === "book-distributed")?.studyStatus).toBe("paused");
    expect(state.books.find((book) => book.id === "book-domain")?.studyStatus).toBe("active");
    expect(state.books.find((book) => book.id === "book-domain")?.studyCycles).toHaveLength(1);
  });

  it("marks completed reading as ready for the study retrospective", async () => {
    const state = await browserWorkspaceCommands.execute({ kind: "completeReading", bookId: "book-distributed" });

    expect(state.books.find((book) => book.id === "book-distributed")).toMatchObject({
      readingCompleted: true,
      studyStatus: "readyToComplete",
    });
  });

  it("permanently removes a book and its learning dependencies", async () => {
    const state = await browserWorkspaceCommands.deleteBook("book-distributed");

    expect(state.books).not.toContainEqual(expect.objectContaining({ id: "book-distributed" }));
    expect(state.drafts).not.toContainEqual(expect.objectContaining({ bookId: "book-distributed" }));
    expect(state.ideas).not.toContainEqual(expect.objectContaining({ bookId: "book-distributed" }));
    expect(state.activeStudyBookId).toBeNull();
  });

  it("persists idea edits, confirmed links, and review decisions", async () => {
    await browserWorkspaceCommands.execute({
      kind: "updateIdea",
      ideaId: "idea-leader",
      formulation: "Смена лидера — наблюдаемый доменный переход с отдельными отказами.",
      assignments: ["recall", "transfer"],
    });
    await browserWorkspaceCommands.execute({
      kind: "linkIdeas",
      fromIdeaId: "idea-leader",
      toIdeaId: "idea-quorum",
      relation: "clarifies",
    });
    await browserWorkspaceCommands.runReview("idea-leader", "ideaReview", "approved");
    const state = await browserWorkspaceCommands.execute({
      kind: "resolveReview",
      ideaId: "idea-leader",
      requestKind: "ideaReview",
      decision: "unchanged",
      formulation: "",
      conclusion: "Ограничение уже отражено в формулировке.",
    });

    expect(state.ideas.find((idea) => idea.id === "idea-leader")).toMatchObject({
      formulation: "Смена лидера — наблюдаемый доменный переход с отдельными отказами.",
      assignments: ["recall", "transfer"],
      versions: expect.arrayContaining([
        expect.objectContaining({ formulation: "Смена лидера — наблюдаемый доменный переход с отдельными отказами." }),
      ]),
    });
    expect(state.ideaLinks).toContainEqual(
      expect.objectContaining({ fromIdeaId: "idea-leader", toIdeaId: "idea-quorum", relation: "clarifies" }),
    );
    expect(state.reviews.at(-1)).toMatchObject({
      ideaId: "idea-leader",
      decision: "unchanged",
      conclusion: "Ограничение уже отражено в формулировке.",
      pending: false,
      response: "",
    });
  });

  it.each([
    ["codex-no-login", "Войдите в Codex"],
    ["codex-crash", "Codex завершился с ошибкой"],
    ["codex-cancel", "Проверка Codex отменена"],
  ] as const)("models %s without invoking a real Codex process", async (scenario, message) => {
    window.__BOOKSHELF_TEST__!.scenario = scenario;

    await expect(browserWorkspaceCommands.runReview("idea-leader", "ideaReview", "approved")).rejects.toThrow(message);
  });

  it("models a local library load failure", async () => {
    window.__BOOKSHELF_TEST__!.scenario = "error";

    await expect(browserWorkspaceCommands.load()).rejects.toThrow("Тестовая библиотека недоступна");
  });

  it("preserves every source fragment when a draft becomes an idea", async () => {
    const initial = await browserWorkspaceCommands.load();
    const draft = initial.drafts.find((item) => item.fragments.length === 2);
    expect(draft).toBeDefined();

    const state = await browserWorkspaceCommands.execute({
      kind: "resolveDraftAsIdea",
      draftId: draft!.id,
      formulation: "Транзакция скрывает несколько разных классов конкурентных отказов.",
      section: draft!.section,
      assignments: ["recall"],
    });

    expect(state.ideas[0]?.fragments).toEqual(draft!.fragments);
    expect(state.drafts).not.toContainEqual(expect.objectContaining({ id: draft!.id }));
    expect(state.milestones.slice(-2).map((milestone) => milestone.kind)).toEqual(["draftResolved", "ideaFormulated"]);
  });

  it("rejects links between ideas owned by different books", async () => {
    await expect(
      browserWorkspaceCommands.execute({
        kind: "linkIdeas",
        fromIdeaId: "idea-leader",
        toIdeaId: "idea-model",
        relation: "complements",
      }),
    ).rejects.toThrow("только идеи одной книги");
  });

  it("keeps the farthest page when the reader returns to an earlier source", async () => {
    await browserWorkspaceCommands.execute({
      kind: "updateReading",
      bookId: "book-distributed",
      page: 330,
      zoom: 1.2,
      scroll: 0.4,
    });
    const state = await browserWorkspaceCommands.execute({
      kind: "updateReading",
      bookId: "book-distributed",
      page: 120,
      zoom: 1.2,
      scroll: 0.1,
    });

    expect(state.books.find((book) => book.id === "book-distributed")?.farthestPage).toBe(330);
  });

  it("requires repeat study before a completed book accepts a new draft", async () => {
    await expect(
      browserWorkspaceCommands.execute({
        kind: "captureDraft",
        bookId: "book-refactoring",
        section: "Глава 2",
        page: 47,
        excerpt: "small steps",
        context: "Refactoring workflow",
        comment: "",
      }),
    ).rejects.toThrow("Начните повторное изучение");

    await browserWorkspaceCommands.execute({ kind: "startRepeatStudy", bookId: "book-refactoring" });
    const state = await browserWorkspaceCommands.execute({
      kind: "captureDraft",
      bookId: "book-refactoring",
      section: "Глава 2",
      page: 47,
      excerpt: "small steps",
      context: "Refactoring workflow",
      comment: "",
    });

    expect(state.drafts).toContainEqual(expect.objectContaining({ bookId: "book-refactoring" }));
  });

  it("creates an experiment intent and enforces the closed lifecycle", async () => {
    let state = await browserWorkspaceCommands.execute({
      kind: "createExperiment",
      ideaId: "idea-model",
      situation: "Моделирование нового контекста",
      action: "Проверить границу на реальном решении",
      nextStep: "Собрать обратную связь",
    });
    const experiment = state.experiments.find((item) => item.ideaId === "idea-model")!;
    expect(experiment.status).toBe("intent");

    await expect(
      browserWorkspaceCommands.execute({
        kind: "advanceExperiment",
        experimentId: experiment.id,
        status: "completed",
        situation: experiment.situation,
        action: experiment.action,
        result: "Результат",
        conclusion: "Вывод",
        cancellationReason: "",
        nextStep: "",
      }),
    ).rejects.toThrow("следующий допустимый этап");

    state = await browserWorkspaceCommands.execute({
      kind: "advanceExperiment",
      experimentId: experiment.id,
      status: "running",
      situation: experiment.situation,
      action: experiment.action,
      result: "",
      conclusion: "",
      cancellationReason: "",
      nextStep: experiment.nextStep,
    });
    expect(state.experiments.find((item) => item.id === experiment.id)?.status).toBe("running");
  });
});
