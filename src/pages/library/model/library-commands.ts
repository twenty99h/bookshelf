import type { CodexStreamEvent, LibraryAction, LibraryState, ReviewKind, SearchResult } from "@/shared/api";

export type StopListening = () => void;

export interface LibraryCommands {
  load(): Promise<LibraryState>;
  execute(action: LibraryAction): Promise<LibraryState>;
  importPdf(): Promise<LibraryState | null>;
  search(query: string): Promise<SearchResult[]>;
  bookUrl(bookId: string): Promise<string>;
  exportArchive(password: string): Promise<boolean>;
  importArchive(password: string): Promise<LibraryState | null>;
  restoreLatestSnapshot(): Promise<LibraryState>;
  exportMaterial(materialId: string, title: string): Promise<boolean>;
  exportDraft(draftId: string): Promise<LibraryState | null>;
  installSignedUpdate(): Promise<boolean>;
  prepareCodexReview(ideaId: string, requestKind: ReviewKind, recallAnswer?: string): Promise<string>;
  runCodexReview(
    requestId: string,
    ideaId: string,
    requestKind: ReviewKind,
    approvedPackage: string,
    recallAnswer?: string,
  ): Promise<LibraryState>;
  cancelCodexReview(requestId: string): Promise<void>;
  startCodexLogin(): Promise<void>;
  openExternalUrl(url: string): Promise<void>;
  onCodexLogin(listener: (event: CodexStreamEvent) => void): Promise<StopListening>;
  onCodexReview(listener: (event: CodexStreamEvent) => void): Promise<StopListening>;
}

export type LibraryView = "library" | "queue" | "ideas" | "study";
