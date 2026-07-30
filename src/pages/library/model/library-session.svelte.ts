import type { LibraryAction, LibraryState } from "@/shared/api";
import { commandErrorMessage } from "@/shared/api";
import type { LibraryCommands, LibraryView } from "./library-commands";

export class LibrarySession {
  library = $state.raw<LibraryState | null>(null);
  view = $state<LibraryView>("library");
  loading = $state(true);
  busy = $state(false);
  error = $state("");
  feedback = $state("");

  readonly #getCommands: () => LibraryCommands;
  #mutationQueue: Promise<void> = Promise.resolve();
  #issuedSnapshot = 0;
  #appliedSnapshot = 0;
  #pendingMutations = 0;

  constructor(getCommands: () => LibraryCommands) {
    this.#getCommands = getCommands;
  }

  get commands(): LibraryCommands {
    return this.#getCommands();
  }

  async load(): Promise<void> {
    const order = this.#nextSnapshotOrder();
    this.loading = true;
    this.error = "";
    try {
      this.#replaceSnapshot(await this.commands.load(), order);
    } catch (cause) {
      this.error = commandErrorMessage(cause);
    } finally {
      this.loading = false;
    }
  }

  async execute(action: LibraryAction, success = "Изменения сохранены"): Promise<boolean> {
    const order = this.#nextSnapshotOrder();
    this.#pendingMutations += 1;
    this.busy = true;
    this.error = "";
    this.feedback = "";

    let succeeded = false;
    const operation = this.#mutationQueue.then(async () => {
      try {
        this.#replaceSnapshot(await this.commands.execute(action), order);
        this.feedback = success;
        succeeded = true;
      } catch (cause) {
        this.error = commandErrorMessage(cause);
      } finally {
        this.#pendingMutations -= 1;
        this.busy = this.#pendingMutations > 0;
      }
    });
    this.#mutationQueue = operation;
    await operation;
    return succeeded;
  }

  replaceFrom(snapshot: LibraryState, order = this.#nextSnapshotOrder()): void {
    this.#replaceSnapshot(snapshot, order);
  }

  beginSnapshotRequest(): number {
    return this.#nextSnapshotOrder();
  }

  navigate(view: LibraryView): void {
    this.view = view;
    this.feedback = "";
    this.error = "";
  }

  #nextSnapshotOrder(): number {
    this.#issuedSnapshot += 1;
    return this.#issuedSnapshot;
  }

  #replaceSnapshot(snapshot: LibraryState, order: number): void {
    if (order < this.#appliedSnapshot) return;
    this.#appliedSnapshot = order;
    this.library = snapshot;
  }
}
