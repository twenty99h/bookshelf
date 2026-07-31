import { convertFileSrc } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import {
  bookFilePath,
  exportDraftMarkdown,
  exportLibraryArchive,
  executeLibraryAction,
  importPdf,
  importLibraryArchive,
  loadLibrary,
  prepareCodexReview,
  restoreLatestSnapshot,
  runCodexReview,
  searchLibrary,
  type LibraryAction,
  type LibraryState,
  type ImportPdfResult,
  type SearchResult,
  type ReviewKind,
} from "@/shared/api";

export interface WorkspaceCommands {
  load(): Promise<LibraryState>;
  execute(action: LibraryAction): Promise<LibraryState>;
  search(query: string): Promise<SearchResult[]>;
  importPdf(): Promise<ImportPdfResult | null>;
  bookUrl(bookId: string): Promise<string | null>;
  exportDraft(draftId: string): Promise<LibraryState | null>;
  prepareReview(ideaId: string, kind: ReviewKind): Promise<string>;
  runReview(ideaId: string, kind: ReviewKind, approvedPackage: string): Promise<LibraryState>;
  restoreBackup(): Promise<LibraryState>;
  exportArchive(password: string): Promise<boolean>;
  importArchive(password: string): Promise<LibraryState | null>;
}

const desktopCommands: WorkspaceCommands = {
  load: loadLibrary,
  execute: executeLibraryAction,
  search: searchLibrary,
  async importPdf() {
    const path = await open({ multiple: false, filters: [{ name: "PDF", extensions: ["pdf"] }] });
    return path ? importPdf(path) : null;
  },
  async bookUrl(bookId) {
    return convertFileSrc(await bookFilePath(bookId));
  },
  async exportDraft(draftId) {
    const path = await save({ filters: [{ name: "Markdown", extensions: ["md"] }] });
    return path ? exportDraftMarkdown(draftId, path) : null;
  },
  prepareReview: (ideaId, kind) => prepareCodexReview(ideaId, kind),
  runReview(ideaId, kind, approvedPackage) {
    return runCodexReview(crypto.randomUUID(), ideaId, kind, approvedPackage);
  },
  restoreBackup: restoreLatestSnapshot,
  async exportArchive(password) {
    const path = await save({ filters: [{ name: "Bookshelf archive", extensions: ["bookshelf"] }] });
    if (!path) return false;
    await exportLibraryArchive(path, password);
    return true;
  },
  async importArchive(password) {
    const path = await open({ multiple: false, filters: [{ name: "Bookshelf archive", extensions: ["bookshelf"] }] });
    return path ? importLibraryArchive(path, password) : null;
  },
};

export async function createWorkspaceCommands(): Promise<WorkspaceCommands> {
  if (__BOOKSHELF_BROWSER_ADAPTER__) {
    const { browserWorkspaceCommands } = await import("./browser-workspace-commands");
    return browserWorkspaceCommands;
  }
  return desktopCommands;
}
