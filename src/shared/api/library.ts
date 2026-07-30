import { invoke } from "@tauri-apps/api/core";
import type { Book } from "./generated/Book";
import type { CodexStreamEvent } from "./generated/CodexStreamEvent";
import type { CommandError } from "./generated/CommandError";
import type { DraftNote } from "./generated/DraftNote";
import type { Experiment } from "./generated/Experiment";
import type { Idea } from "./generated/Idea";
import type { IdeaAssignment } from "./generated/IdeaAssignment";
import type { IdeaLink } from "./generated/IdeaLink";
import type { IdeaRelation } from "./generated/IdeaRelation";
import type { IdeaReview } from "./generated/IdeaReview";
import type { LibraryAction } from "./generated/LibraryAction";
import type { LibraryState } from "./generated/LibraryState";
import type { OutlineItem } from "./generated/OutlineItem";
import type { Recall } from "./generated/Recall";
import type { RecallRating } from "./generated/RecallRating";
import type { Retrospective } from "./generated/Retrospective";
import type { ReviewDecision } from "./generated/ReviewDecision";
import type { ReviewKind } from "./generated/ReviewKind";
import type { SearchResult } from "./generated/SearchResult";
import type { SearchResultKind } from "./generated/SearchResultKind";
import type { StudySession } from "./generated/StudySession";
import type { SessionStatus } from "./generated/SessionStatus";
import type { Topic } from "./generated/Topic";
import type { TransferMaterial } from "./generated/TransferMaterial";

export type {
  Book,
  CodexStreamEvent,
  CommandError,
  DraftNote,
  Experiment,
  Idea,
  IdeaAssignment,
  IdeaLink,
  IdeaRelation,
  IdeaReview,
  LibraryAction,
  LibraryState,
  OutlineItem,
  Recall,
  RecallRating,
  Retrospective,
  ReviewDecision,
  ReviewKind,
  SearchResult,
  SearchResultKind,
  StudySession,
  SessionStatus,
  Topic,
  TransferMaterial,
};
export async function loadLibrary(): Promise<LibraryState> {
  return invoke<LibraryState>("load_library");
}
export async function executeLibraryAction(action: LibraryAction): Promise<LibraryState> {
  return invoke<LibraryState>("execute_library_action", { action });
}
export async function importPdf(path: string, title = ""): Promise<LibraryState> {
  return invoke<LibraryState>("import_pdf", { path, title });
}
export async function searchLibrary(query: string): Promise<SearchResult[]> {
  return invoke<SearchResult[]>("search_library", { query });
}
export async function bookFilePath(bookId: string): Promise<string> {
  return invoke<string>("book_file_path", { bookId });
}
export async function exportLibraryArchive(path: string, password: string): Promise<void> {
  return invoke("export_library_archive", { path, password });
}
export async function importLibraryArchive(path: string, password: string): Promise<LibraryState> {
  return invoke<LibraryState>("import_library_archive", { path, password });
}
export async function restoreLatestSnapshot(): Promise<LibraryState> {
  return invoke<LibraryState>("restore_latest_snapshot");
}
export async function exportMaterialMarkdown(materialId: string, path: string): Promise<void> {
  return invoke("export_material_markdown", { materialId, path });
}
export async function exportDraftMarkdown(draftId: string, path: string): Promise<LibraryState> {
  return invoke<LibraryState>("export_draft_markdown", { draftId, path });
}
export async function installSignedUpdate(): Promise<boolean> {
  return invoke<boolean>("install_signed_update");
}
export async function prepareCodexReview(
  ideaId: string,
  requestKind: ReviewKind,
  recallAnswer?: string,
): Promise<string> {
  return invoke<string>("prepare_codex_review", { ideaId, requestKind, recallAnswer });
}
export async function runCodexReview(
  requestId: string,
  ideaId: string,
  requestKind: ReviewKind,
  approvedPackage: string,
  recallAnswer?: string,
): Promise<LibraryState> {
  return invoke<LibraryState>("run_codex_review", {
    requestId,
    ideaId,
    requestKind,
    recallAnswer,
    approvedPackage,
  });
}
export async function cancelCodexReview(requestId: string): Promise<void> {
  return invoke("cancel_codex_review", { requestId });
}
export async function startCodexLogin(): Promise<void> {
  return invoke("start_codex_login");
}

export function commandErrorMessage(cause: unknown): string {
  if (typeof cause === "object" && cause !== null) {
    const error = cause as CommandError;
    if (typeof error.message === "string") return error.message;
  }
  return String(cause);
}
