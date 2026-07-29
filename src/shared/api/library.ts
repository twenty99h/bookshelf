import { invoke } from "@tauri-apps/api/core";
import type { Book } from "./generated/Book";
import type { CodexStreamEvent } from "./generated/CodexStreamEvent";
import type { DraftNote } from "./generated/DraftNote";
import type { Experiment } from "./generated/Experiment";
import type { Idea } from "./generated/Idea";
import type { IdeaLink } from "./generated/IdeaLink";
import type { IdeaReview } from "./generated/IdeaReview";
import type { LibraryAction } from "./generated/LibraryAction";
import type { LibraryState } from "./generated/LibraryState";
import type { OutlineItem } from "./generated/OutlineItem";
import type { Recall } from "./generated/Recall";
import type { Retrospective } from "./generated/Retrospective";
import type { ReviewDecision } from "./generated/ReviewDecision";
import type { ReviewKind } from "./generated/ReviewKind";
import type { SearchResult } from "./generated/SearchResult";
import type { StudySession } from "./generated/StudySession";
import type { Topic } from "./generated/Topic";
import type { TransferMaterial } from "./generated/TransferMaterial";

export type {
  Book, CodexStreamEvent, DraftNote, Experiment, Idea, IdeaLink, IdeaReview, LibraryAction,
  LibraryState, OutlineItem, Recall, Retrospective, ReviewDecision, ReviewKind,
  SearchResult, StudySession, Topic, TransferMaterial,
};
type CommandError = { code?: string; message?: string };

const emptyFields: Omit<LibraryState, "books" | "workspaceNote"> = {
  drafts: [], ideas: [], topics: [], ideaLinks: [], experiments: [], recalls: [], sessions: [], materials: [], reviews: [],
  activeStudyBookId: null, weeklySessionBudget: 3, lastDebtChange: 0, lastDebtChangedAt: 0, debtNotificationSentAt: null, debtReminderDays: 7,
};

function normalizeLibrary(value: Partial<LibraryState>): LibraryState {
  return { ...emptyFields, books: [], workspaceNote: "", ...value };
}

export async function loadLibrary(): Promise<LibraryState> { return normalizeLibrary(await invoke<LibraryState>("load_library")); }
export async function saveWorkspaceNote(note: string): Promise<LibraryState> { return normalizeLibrary(await invoke<LibraryState>("save_workspace_note", { note })); }
export async function executeLibraryAction(action: LibraryAction): Promise<LibraryState> { return normalizeLibrary(await invoke<LibraryState>("execute_library_action", { action })); }
export async function importPdf(path: string, title = ""): Promise<LibraryState> { return normalizeLibrary(await invoke<LibraryState>("import_pdf", { path, title })); }
export async function searchLibrary(query: string): Promise<SearchResult[]> { return invoke<SearchResult[]>("search_library", { query }); }
export async function bookFilePath(bookId: string): Promise<string> { return invoke<string>("book_file_path", { bookId }); }
export async function exportLibraryArchive(path: string, password: string): Promise<void> { return invoke("export_library_archive", { path, password }); }
export async function importLibraryArchive(path: string, password: string): Promise<LibraryState> { return normalizeLibrary(await invoke<LibraryState>("import_library_archive", { path, password })); }
export async function restoreLatestSnapshot(): Promise<LibraryState> { return normalizeLibrary(await invoke<LibraryState>("restore_latest_snapshot")); }
export async function exportMaterialMarkdown(materialId: string, path: string): Promise<void> { return invoke("export_material_markdown", { materialId, path }); }
export async function exportDraftMarkdown(draftId: string, path: string): Promise<LibraryState> { return normalizeLibrary(await invoke<LibraryState>("export_draft_markdown", { draftId, path })); }
export async function installSignedUpdate(): Promise<boolean> { return invoke<boolean>("install_signed_update"); }
export async function runCodexReview(requestId: string, ideaId: string, requestKind: ReviewKind, reviewPackage: string): Promise<LibraryState> {
  return normalizeLibrary(await invoke<LibraryState>("run_codex_review", { requestId, ideaId, requestKind, package: reviewPackage }));
}
export async function cancelCodexReview(requestId: string): Promise<void> { return invoke("cancel_codex_review", { requestId }); }
export async function startCodexLogin(): Promise<void> { return invoke("start_codex_login"); }

export function commandErrorMessage(cause: unknown): string {
  if (typeof cause === "object" && cause !== null) {
    const error = cause as CommandError;
    if (typeof error.message === "string") return error.message;
  }
  return String(cause);
}
