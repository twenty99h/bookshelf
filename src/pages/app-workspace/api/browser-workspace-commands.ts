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
    return results;
  },
  async importPdf() {
    if (state.books.length === 0) state = activeLibraryFixture();
    commands.push({ kind: "importPdf" });
    return persist();
  },
  async bookUrl() {
    return null;
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
        book.studyStatus = "archived";
        if (state.activeStudyBookId === book.id) state.activeStudyBookId = null;
      }
      break;
    }
    case "restoreBook": {
      const book = state.books.find((item) => item.id === action.bookId);
      if (book) {
        book.archived = false;
        book.studyStatus = book.studyCompleted ? "completed" : "paused";
      }
      break;
    }
  }
}
