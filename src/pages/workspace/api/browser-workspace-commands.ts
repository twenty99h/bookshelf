import type { LibraryAction, LibraryState, SearchResult } from "@/shared/api";
import { activeLibraryFixture, emptyLibraryFixture } from "../model/workspace-fixtures";
import type { WorkspaceCommands } from "./workspace-commands";

const storageKey = "bookshelf-browser-fixture";
let state = readInitialState();
const commands: unknown[] = [];
type BrowserScenario = NonNullable<Window["__BOOKSHELF_TEST__"]>["scenario"];
let scenario = (new URLSearchParams(location.search).get("scenario") as BrowserScenario | null) ?? "success";
let archiveAt: number | null = null;

function clone<T>(value: T): T {
  return structuredClone(value);
}

function persist(): LibraryState {
  sessionStorage.setItem(storageKey, JSON.stringify(state));
  return clone(state);
}

function readInitialState(): LibraryState {
  const saved = sessionStorage.getItem(storageKey);
  if (saved) return JSON.parse(saved) as LibraryState;
  return new URLSearchParams(location.search).get("fixture") === "empty"
    ? emptyLibraryFixture()
    : activeLibraryFixture();
}

function exposeHarness() {
  window.__BOOKSHELF_TEST__ = {
    commands,
    get scenario() {
      return scenario;
    },
    set scenario(value) {
      scenario = value;
    },
    reset(fixture = "active") {
      state = fixture === "empty" ? emptyLibraryFixture() : activeLibraryFixture();
      commands.length = 0;
      scenario = "success";
      archiveAt = null;
      persist();
    },
  };
}

exposeHarness();

export const browserWorkspaceCommands: WorkspaceCommands = {
  async load() {
    if (scenario === "loading") return new Promise<LibraryState>(() => {});
    if (scenario === "error") throw new Error("Тестовая библиотека недоступна");
    return clone(state);
  },
  async execute(action) {
    commands.push(clone(action));
    applyAction(action);
    return persist();
  },
  async search(query) {
    const needle = query.trim().toLocaleLowerCase("ru");
    if (!needle) return [];
    const results: SearchResult[] = [];
    for (const book of state.books) {
      if (book.title.toLocaleLowerCase("ru").includes(needle))
        results.push({ id: book.id, kind: "book", title: book.title, context: "Книга" });
    }
    for (const idea of state.ideas) {
      if (idea.formulation.toLocaleLowerCase("ru").includes(needle))
        results.push({ id: idea.id, kind: "idea", title: idea.formulation, context: "Идея книги" });
    }
    for (const draft of state.drafts) {
      if (`${draft.excerpt} ${draft.comment}`.toLocaleLowerCase("ru").includes(needle))
        results.push({ id: draft.id, kind: "draft", title: draft.excerpt, context: "Черновая заметка" });
    }
    for (const topic of state.topics) {
      if (topic.name.toLocaleLowerCase("ru").includes(needle))
        results.push({ id: topic.id, kind: "topic", title: topic.name, context: "Тема знаний" });
    }
    for (const material of state.materials) {
      if (`${material.title} ${material.idea}`.toLocaleLowerCase("ru").includes(needle))
        results.push({ id: material.id, kind: "material", title: material.title, context: "Материал для передачи" });
    }
    return results;
  },
  async importPdf() {
    if (state.books.length === 0) state = activeLibraryFixture();
    commands.push({ kind: "importPdf" });
    return { state: persist(), bookId: state.books[0]?.id ?? "", duplicate: false };
  },
  async bookUrl() {
    return "/bookshelf-test.pdf";
  },
  async deleteBook(bookId) {
    commands.push({ kind: "deleteBook", bookId });
    applyAction({ kind: "deleteBook", bookId });
    return persist();
  },
  async exportDraft(draftId) {
    commands.push({ kind: "exportDraft", draftId });
    applyAction({ kind: "discardDraft", draftId });
    return persist();
  },
  async prepareReview(ideaId, kind) {
    commands.push({ kind: "prepareReview", ideaId, requestKind: kind });
    const idea = state.ideas.find((item) => item.id === ideaId);
    return `Инструкция: проверь ограничения.\n\nИсточник: ${idea?.fragments[0]?.excerpt ?? ""}\n\nАвторская формулировка: ${idea?.formulation ?? ""}`;
  },
  async runReview(ideaId, kind, approvedPackage) {
    commands.push({ kind: "runReview", ideaId, requestKind: kind, approvedPackage });
    if (scenario === "codex-no-login") throw new Error("Войдите в Codex, затем повторите явную проверку");
    if (scenario === "codex-crash") throw new Error("Codex завершился с ошибкой; пакет можно перенести вручную");
    if (scenario === "codex-cancel") throw new Error("Проверка Codex отменена; идея не изменена");
    state.reviews.push({
      id: `review-${state.reviews.length + 1}`,
      ideaId,
      requestKind: kind,
      response: "Проверьте границы применимости и сценарий смены лидера.",
      pending: true,
      decision: "pending",
      conclusion: "",
      reviewedAt: 1_785_283_200,
    });
    return persist();
  },
  async restoreBackup() {
    commands.push({ kind: "restoreBackup" });
    return persist();
  },
  async exportArchive() {
    commands.push({ kind: "exportArchive" });
    archiveAt = 1_785_283_200;
    return true;
  },
  async importArchive() {
    commands.push({ kind: "importArchive" });
    return persist();
  },
  async checkForUpdate() {
    commands.push({ kind: "checkForUpdate" });
    return false;
  },
  async exportDiagnostics(entries) {
    commands.push({ kind: "exportDiagnostics", entries: entries.slice(-100) });
    return true;
  },
  async backupMetadata() {
    return { snapshotAt: 1_785_283_200, archiveAt };
  },
};

function applyAction(action: LibraryAction) {
  switch (action.kind) {
    case "updateReading": {
      const book = state.books.find((item) => item.id === action.bookId);
      if (book) {
        book.reading = { page: action.page, zoom: action.zoom, scroll: action.scroll };
        book.farthestPage = Math.max(book.farthestPage, action.page);
      }
      break;
    }
    case "updateReaderPreferences": {
      const book = state.books.find((item) => item.id === action.bookId);
      if (book) book.reader = action.preferences;
      break;
    }
    case "captureDraft":
      ensureBookAllowsCapture(action.bookId);
      state.drafts.unshift({
        id: `draft-${String(state.drafts.length + 1).padStart(3, "0")}`,
        bookId: action.bookId,
        section: action.section,
        page: action.page,
        excerpt: action.excerpt,
        context: action.context,
        comment: action.comment,
        fragments: [{ page: action.page, excerpt: action.excerpt, context: action.context }],
        createdAt: 1_785_283_200,
      });
      break;
    case "captureDraftSources": {
      ensureBookAllowsCapture(action.bookId);
      const first = action.fragments[0];
      if (!first) break;
      state.drafts.unshift({
        id: `draft-${String(state.drafts.length + 1).padStart(3, "0")}`,
        bookId: action.bookId,
        section: action.section,
        page: first.page,
        excerpt: first.excerpt,
        context: first.context,
        comment: action.comment,
        fragments: clone(action.fragments),
        createdAt: 1_785_283_200,
      });
      break;
    }
    case "resolveDraftAsIdea": {
      const draft = state.drafts.find((item) => item.id === action.draftId);
      if (!draft) break;
      state.ideas.unshift({
        id: `idea-${state.ideas.length + 1}`,
        bookId: draft.bookId,
        section: action.section,
        formulation: action.formulation,
        assignments: action.assignments,
        fragments: clone(draft.fragments),
        versions: [],
        topicIds: [],
      });
      state.drafts = state.drafts.filter((item) => item.id !== action.draftId);
      state.milestones.push(
        {
          id: `milestone-draft-resolved-${state.milestones.length + 1}`,
          bookId: draft.bookId,
          kind: "draftResolved",
          occurredAt: 1_785_283_200,
          page: null,
        },
        {
          id: `milestone-idea-formulated-${state.milestones.length + 2}`,
          bookId: draft.bookId,
          kind: "ideaFormulated",
          occurredAt: 1_785_283_200,
          page: null,
        },
      );
      break;
    }
    case "attachDraftToIdea": {
      const draft = state.drafts.find((item) => item.id === action.draftId);
      const idea = state.ideas.find((item) => item.id === action.ideaId);
      if (draft && idea && draft.bookId === idea.bookId) idea.fragments.push(...clone(draft.fragments));
      if (draft && idea) state.drafts = state.drafts.filter((item) => item.id !== draft.id);
      break;
    }
    case "deferDraft": {
      const index = state.drafts.findIndex((item) => item.id === action.draftId);
      if (index >= 0) state.drafts.push(...state.drafts.splice(index, 1));
      break;
    }
    case "discardDraft":
      state.drafts = state.drafts.filter((item) => item.id !== action.draftId);
      break;
    case "activateStudy": {
      for (const book of state.books) {
        if (book.id === state.activeStudyBookId && book.id !== action.bookId) book.studyStatus = "paused";
      }
      const book = state.books.find((item) => item.id === action.bookId);
      if (book) {
        book.studyStatus = book.studyCycles.length > 0 ? "repeating" : "active";
        if (book.studyCycles.length === 0) {
          book.studyCycles.push({
            id: `study-cycle-${book.id}`,
            startedAt: 1_785_283_200,
            completedAt: null,
            retrospective: null,
          });
        }
      }
      state.activeStudyBookId = action.bookId;
      break;
    }
    case "completeReading": {
      const book = state.books.find((item) => item.id === action.bookId);
      if (book) {
        book.readingCompleted = true;
        book.studyStatus = "readyToComplete";
      }
      break;
    }
    case "startRepeatStudy": {
      const book = state.books.find((item) => item.id === action.bookId);
      if (book) {
        book.studyStatus = "repeating";
        book.studyCycles.push({
          id: `study-cycle-${book.id}-${book.studyCycles.length + 1}`,
          startedAt: 1_785_283_200,
          completedAt: null,
          retrospective: null,
        });
        state.activeStudyBookId = book.id;
      }
      break;
    }
    case "archiveBook": {
      const book = state.books.find((item) => item.id === action.bookId);
      if (book) {
        book.archived = true;
        if (state.activeStudyBookId === book.id) {
          book.studyStatus = "paused";
          state.activeStudyBookId = null;
        }
      }
      break;
    }
    case "restoreBook": {
      const book = state.books.find((item) => item.id === action.bookId);
      if (book) {
        book.archived = false;
      }
      break;
    }
    case "updateIdea": {
      const idea = state.ideas.find((item) => item.id === action.ideaId);
      if (idea) {
        if (idea.formulation !== action.formulation) {
          idea.versions.push({ formulation: action.formulation, savedAt: 1_785_283_200 });
        }
        idea.formulation = action.formulation;
        idea.assignments = clone(action.assignments);
      }
      break;
    }
    case "linkIdeas": {
      const from = state.ideas.find((idea) => idea.id === action.fromIdeaId);
      const to = state.ideas.find((idea) => idea.id === action.toIdeaId);
      if (from && to && from.bookId !== to.bookId) throw new Error("Связывать можно только идеи одной книги");
      state.ideaLinks.push({
        id: `link-${state.ideaLinks.length + 1}`,
        fromIdeaId: action.fromIdeaId,
        toIdeaId: action.toIdeaId,
        relation: action.relation,
      });
      break;
    }
    case "resolveReview": {
      const pending = state.reviews.find(
        (review) => review.ideaId === action.ideaId && review.requestKind === action.requestKind && review.pending,
      );
      if (action.decision === "refined") {
        applyAction({
          kind: "updateIdea",
          ideaId: action.ideaId,
          formulation: action.formulation,
          assignments: state.ideas.find((idea) => idea.id === action.ideaId)?.assignments ?? [],
        });
      }
      state.reviews = state.reviews.filter(
        (review) => !(review.ideaId === action.ideaId && review.requestKind === action.requestKind && review.pending),
      );
      state.reviews.push({
        id: pending?.id ?? `review-${state.reviews.length + 1}`,
        ideaId: action.ideaId,
        requestKind: action.requestKind,
        response: action.decision === "later" ? (pending?.response ?? "") : "",
        pending: action.decision === "later",
        decision: action.decision,
        conclusion: action.conclusion,
        reviewedAt: 1_785_283_200,
      });
      break;
    }
    case "createExperiment": {
      if (!action.situation.trim() || !action.action.trim()) throw new Error("Опишите ситуацию и проверяемое действие");
      state.experiments.push({
        id: `experiment-${state.experiments.length + 1}`,
        ideaId: action.ideaId,
        situation: action.situation,
        action: action.action,
        result: "",
        conclusion: "",
        status: "intent",
        cancellationReason: "",
        nextStep: action.nextStep,
      });
      state.experimentDrafts = state.experimentDrafts.filter((draft) => draft.ideaId !== action.ideaId);
      break;
    }
    case "saveExperimentDraft": {
      state.experimentDrafts = [
        ...state.experimentDrafts.filter((draft) => draft.id !== action.draft.id),
        structuredClone(action.draft),
      ];
      break;
    }
    case "deleteBook": {
      const ideaIds = new Set(state.ideas.filter((idea) => idea.bookId === action.bookId).map((idea) => idea.id));
      state.books = state.books.filter((book) => book.id !== action.bookId);
      state.drafts = state.drafts.filter((draft) => draft.bookId !== action.bookId);
      state.ideas = state.ideas.filter((idea) => idea.bookId !== action.bookId);
      state.ideaLinks = state.ideaLinks.filter((link) => !ideaIds.has(link.fromIdeaId) && !ideaIds.has(link.toIdeaId));
      state.experiments = state.experiments.filter((item) => !ideaIds.has(item.ideaId));
      state.experimentDrafts = state.experimentDrafts.filter((item) => !ideaIds.has(item.ideaId));
      state.recalls = state.recalls.filter((item) => !ideaIds.has(item.ideaId));
      state.reviews = state.reviews.filter((item) => !ideaIds.has(item.ideaId));
      state.materials = state.materials
        .map((material) => ({ ...material, ideaIds: material.ideaIds.filter((id) => !ideaIds.has(id)) }))
        .filter((material) => material.ideaIds.length > 0);
      state.milestones = state.milestones.filter((item) => item.bookId !== action.bookId);
      state.completionDrafts = state.completionDrafts.filter((item) => item.bookId !== action.bookId);
      if (state.activeStudyBookId === action.bookId) state.activeStudyBookId = null;
      break;
    }
    case "completeRecall": {
      const recall = state.recalls.find((item) => item.id === action.recallId);
      if (recall) {
        recall.answer = action.answer;
        recall.rating = action.rating;
        recall.nextAt = action.nextAt ?? 1_785_456_000;
      }
      break;
    }
    case "rescheduleRecall": {
      const recall = state.recalls.find((item) => item.id === action.recallId);
      if (recall) recall.nextAt = action.nextAt;
      break;
    }
    case "advanceExperiment": {
      const experiment = state.experiments.find((item) => item.id === action.experimentId);
      if (experiment) {
        if (!validExperimentTransition(experiment.status, action.status)) {
          throw new Error("Выберите следующий допустимый этап эксперимента");
        }
        experiment.status = action.status;
        experiment.situation = action.situation;
        experiment.action = action.action;
        experiment.result = action.result;
        experiment.conclusion = action.conclusion;
        experiment.cancellationReason = action.cancellationReason;
        experiment.nextStep = action.nextStep;
      }
      break;
    }
    case "saveStudyCompletionDraft":
      state.completionDrafts = [
        ...state.completionDrafts.filter((item) => item.bookId !== action.draft.bookId),
        clone(action.draft),
      ];
      break;
    case "completeStudy": {
      const book = state.books.find((item) => item.id === action.bookId);
      if (book) {
        book.studyStatus = "completed";
        book.retrospective = {
          text: action.retrospective,
          significantIdeaIds: action.significantIdeaIds,
          continuingWork: action.continuingWork,
          unfinishedWorkDecision: action.unfinishedWorkDecision,
          workDecisions: clone(action.workDecisions),
        };
      }
      state.completionDrafts = state.completionDrafts.filter((item) => item.bookId !== action.bookId);
      if (state.activeStudyBookId === action.bookId) state.activeStudyBookId = null;
      break;
    }
  }
}

function validExperimentTransition(from: string, to: string) {
  if (from === to) return !["completed", "cancelled"].includes(from);
  return [
    "intent:running",
    "intent:cancelled",
    "running:reviewing",
    "running:cancelled",
    "reviewing:running",
    "reviewing:completed",
    "reviewing:cancelled",
  ].includes(`${from}:${to}`);
}

function ensureBookAllowsCapture(bookId: string) {
  const book = state.books.find((item) => item.id === bookId);
  if (book?.studyStatus === "completed") {
    throw new Error("Начните повторное изучение, чтобы создавать новые черновые заметки");
  }
}
