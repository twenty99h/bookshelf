import { invoke } from "@tauri-apps/api/core";

export type BookSummary = { id: string; title: string };
export type LibraryState = { books: BookSummary[]; workspaceNote: string };

type CommandError = { code?: string; message?: string };

export async function loadLibrary(): Promise<LibraryState> {
  return invoke<LibraryState>("load_library");
}

export async function saveWorkspaceNote(note: string): Promise<LibraryState> {
  return invoke<LibraryState>("save_workspace_note", { note });
}

export function commandErrorMessage(cause: unknown): string {
  if (typeof cause === "object" && cause !== null) {
    const error = cause as CommandError;
    if (typeof error.message === "string") return error.message;
  }
  return String(cause);
}
