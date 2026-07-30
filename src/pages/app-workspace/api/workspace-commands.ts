import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  bookFilePath,
  executeLibraryAction,
  importPdf,
  loadLibrary,
  searchLibrary,
  type LibraryAction,
  type LibraryState,
  type SearchResult,
} from "@/shared/api";

export interface WorkspaceCommands {
  load(): Promise<LibraryState>;
  execute(action: LibraryAction): Promise<LibraryState>;
  search(query: string): Promise<SearchResult[]>;
  importPdf(): Promise<LibraryState | null>;
  bookUrl(bookId: string): Promise<string | null>;
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
};

export async function createWorkspaceCommands(): Promise<WorkspaceCommands> {
  if (__BOOKSHELF_BROWSER_ADAPTER__) {
    const { browserWorkspaceCommands } = await import("./browser-workspace-commands");
    return browserWorkspaceCommands;
  }
  return desktopCommands;
}
