import type { LibraryAction, LibraryState, SearchResult } from "@/shared/api";
import { activeLibraryFixture, emptyLibraryFixture } from "../model/workspace-fixtures";
import type { WorkspaceCommands } from "./workspace-commands";

const storageKey = "bookshelf-browser-fixture";
let state = readInitialState();
const commands: unknown[] = [];

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
    scenario: "success",
    reset(fixture = "active") {
      state = fixture === "empty" ? emptyLibraryFixture() : activeLibraryFixture();
      commands.length = 0;
      persist();
    },
  };
}

exposeHarness();

export const browserWorkspaceCommands: WorkspaceCommands = {
  async load() {
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
        results.push({ id: material.id, kind: "material", title: material.title, context: "Материал" });
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
    return true;
  },
  async importArchive() {
    commands.push({ kind: "importArchive" });
    return persist();
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
      if (book) book.studyStatus = book.studyCycles.length > 0 ? "repeating" : "active";
      state.activeStudyBookId = action.bookId;
      break;
    }
    case "completeReading": {
      const book = state.books.find((item) => item.id === action.bookId);
      if (book) book.readingCompleted = true;
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
    case "completeRecall": {
      const recall = state.recalls.find((item) => item.ideaId === action.ideaId);
      if (recall) {
        recall.answer = action.answer;
        recall.rating = action.rating;
        recall.nextAt = action.nextAt ?? 1_785_456_000;
      }
      break;
    }
    case "advanceExperiment": {
      const experiment = state.experiments.find((item) => item.id === action.experimentId);
      if (experiment) {
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
        };
      }
      state.completionDrafts = state.completionDrafts.filter((item) => item.bookId !== action.bookId);
      if (state.activeStudyBookId === action.bookId) state.activeStudyBookId = null;
      break;
    }
  }
}
