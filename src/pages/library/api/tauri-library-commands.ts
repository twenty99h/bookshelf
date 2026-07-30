import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  bookFilePath,
  cancelCodexReview,
  executeLibraryAction,
  exportDraftMarkdown,
  exportLibraryArchive,
  exportMaterialMarkdown,
  importLibraryArchive,
  importPdf,
  installSignedUpdate,
  loadLibrary,
  prepareCodexReview,
  restoreLatestSnapshot,
  runCodexReview,
  searchLibrary,
  startCodexLogin,
  type CodexStreamEvent,
} from "@/shared/api";
import type { LibraryCommands } from "../model/library-commands";

export const tauriLibraryCommands: LibraryCommands = {
  load: loadLibrary,
  execute: executeLibraryAction,
  async importPdf() {
    const path = await open({ multiple: false, filters: [{ name: "PDF", extensions: ["pdf"] }] });
    return path ? importPdf(path) : null;
  },
  search: searchLibrary,
  async bookUrl(bookId) {
    return convertFileSrc(await bookFilePath(bookId));
  },
  async exportArchive(password) {
    const path = await save({
      defaultPath: "bookshelf-library.age",
      filters: [{ name: "Bookshelf archive", extensions: ["age"] }],
    });
    if (!path) return false;
    await exportLibraryArchive(path, password);
    return true;
  },
  async importArchive(password) {
    const path = await open({ multiple: false, filters: [{ name: "Bookshelf archive", extensions: ["age"] }] });
    return path ? importLibraryArchive(path, password) : null;
  },
  restoreLatestSnapshot,
  async exportMaterial(materialId, title) {
    const path = await save({
      defaultPath: `${title || "material"}.md`,
      filters: [{ name: "Markdown", extensions: ["md"] }],
    });
    if (!path) return false;
    await exportMaterialMarkdown(materialId, path);
    return true;
  },
  async exportDraft(draftId) {
    const path = await save({ defaultPath: "draft-note.md", filters: [{ name: "Markdown", extensions: ["md"] }] });
    return path ? exportDraftMarkdown(draftId, path) : null;
  },
  installSignedUpdate,
  prepareCodexReview,
  runCodexReview,
  cancelCodexReview,
  startCodexLogin,
  openExternalUrl: openUrl,
  onCodexLogin(listener) {
    return listen<CodexStreamEvent>("codex-login-event", (event) => listener(event.payload));
  },
  onCodexReview(listener) {
    return listen<CodexStreamEvent>("codex-review-event", (event) => listener(event.payload));
  },
};
