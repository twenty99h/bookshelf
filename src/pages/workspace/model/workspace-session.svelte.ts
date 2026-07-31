import { getContext, setContext } from "svelte";
import { commandErrorMessage, type LibraryAction, type LibraryState } from "@/shared/api";
import { createWorkspaceCommands, type WorkspaceCommands } from "../api/workspace-commands";

const workspaceSessionKey = Symbol("workspace-session");

export class WorkspaceSession {
  commands = $state<WorkspaceCommands | null>(null);
  library = $state.raw<LibraryState | null>(null);
  loading = $state(true);
  busy = $state(false);
  error = $state("");
  feedback = $state("");
  #loadPromise: Promise<void> | null = null;

  load(): Promise<void> {
    if (this.#loadPromise) return this.#loadPromise;
    this.#loadPromise = this.#load();
    return this.#loadPromise;
  }

  async #load() {
    try {
      this.commands = await createWorkspaceCommands();
      this.library = await this.commands.load();
    } catch (cause) {
      this.error = commandErrorMessage(cause);
    } finally {
      this.loading = false;
    }
  }

  async execute(action: LibraryAction, message = "Сохранено"): Promise<boolean> {
    if (!this.commands) return false;
    this.busy = true;
    this.error = "";
    this.feedback = "";
    try {
      this.library = await this.commands.execute(action);
      this.feedback = message;
      return true;
    } catch (cause) {
      this.error = commandErrorMessage(cause);
      return false;
    } finally {
      this.busy = false;
    }
  }

  replaceLibrary(library: LibraryState, feedback = "") {
    this.library = library;
    this.feedback = feedback;
  }
}

export function provideWorkspaceSession(): WorkspaceSession {
  const session = new WorkspaceSession();
  setContext(workspaceSessionKey, session);
  return session;
}

export function useWorkspaceSession(): WorkspaceSession {
  return getContext<WorkspaceSession | undefined>(workspaceSessionKey) ?? new WorkspaceSession();
}
