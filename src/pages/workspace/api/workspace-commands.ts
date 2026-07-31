import { convertFileSrc } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import {
  bookFilePath,
  deleteBook,
  exportDraftMarkdown,
  exportDiagnosticLog,
  exportLibraryArchive,
  executeLibraryAction,
  importPdfFromDialog,
  importLibraryArchive,
  installSignedUpdate,
  loadBackupMetadata,
  loadLibrary,
  prepareCodexReview,
  restoreLatestSnapshot,
  runCodexReview,
  searchLibrary,
  type LibraryAction,
  type BackupMetadata,
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
  deleteBook(bookId: string): Promise<LibraryState>;
  exportDraft(draftId: string): Promise<LibraryState | null>;
  prepareReview(ideaId: string, kind: ReviewKind): Promise<string>;
  runReview(ideaId: string, kind: ReviewKind, approvedPackage: string): Promise<LibraryState>;
  restoreBackup(): Promise<LibraryState>;
  exportArchive(password: string): Promise<boolean>;
  importArchive(password: string): Promise<LibraryState | null>;
  checkForUpdate(): Promise<boolean>;
  exportDiagnostics(entries: string[]): Promise<boolean>;
  backupMetadata(): Promise<BackupMetadata>;
}

const desktopCommands: WorkspaceCommands = {
  load: loadLibrary,
  execute: executeLibraryAction,
  search: searchLibrary,
  importPdf: importPdfFromDialog,
  async bookUrl(bookId) {
    return convertFileSrc(await bookFilePath(bookId));
  },
  deleteBook,
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
  checkForUpdate: installSignedUpdate,
  async exportDiagnostics(entries) {
    const path = await save({ filters: [{ name: "Diagnostic log", extensions: ["log"] }] });
    if (!path) return false;
    await exportDiagnosticLog(path, entries);
    return true;
  },
  backupMetadata: loadBackupMetadata,
};

export async function createWorkspaceCommands(): Promise<WorkspaceCommands> {
  if (__BOOKSHELF_BROWSER_ADAPTER__) {
    const { browserWorkspaceCommands } = await import("./browser-workspace-commands");
    return browserWorkspaceCommands;
  }
  return desktopCommands;
}
