import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { LibraryAction, LibraryState } from "@/shared/api";
import { activeLibraryFixture } from "@/features/workspace";
import WorkspacePage from "./WorkspacePage.svelte";

vi.mock("@lucide/svelte", async () => {
  const { default: Icon } = await import("@/features/workspace/ui/IconStub.test.svelte");
  return {
    ArrowLeft: Icon,
    BookOpen: Icon,
    BookCopy: Icon,
    Brain: Icon,
    Check: Icon,
    ChevronRight: Icon,
    Command: Icon,
    FileArchive: Icon,
    FlaskConical: Icon,
    Gauge: Icon,
    Library: Icon,
    ListTree: Icon,
    Menu: Icon,
    MoreHorizontal: Icon,
    PanelRight: Icon,
    Plus: Icon,
    Search: Icon,
    Settings: Icon,
    Sparkles: Icon,
    StickyNote: Icon,
    X: Icon,
    ZoomIn: Icon,
    ZoomOut: Icon,
  };
});

const execute = vi.fn<(action: LibraryAction) => Promise<LibraryState>>();
const commandFactory = vi.hoisted(() => vi.fn());
let state: LibraryState;

vi.mock("@/features/workspace", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@/features/workspace")>()),
  createWorkspaceCommands: commandFactory,
}));

describe("workspace page behavior", () => {
  beforeEach(() => {
    state = activeLibraryFixture();
    commandFactory.mockResolvedValue({
      load: async () => structuredClone(state),
      execute,
      search: async () => [],
      importPdf: async () => null,
      bookUrl: async () => null,
      exportDraft: async () => null,
      prepareReview: async () => "",
      runReview: async () => structuredClone(state),
      restoreBackup: async () => structuredClone(state),
      exportArchive: async () => true,
      importArchive: async () => structuredClone(state),
    });
    execute.mockImplementation(async (action) => {
      if (action.kind === "discardDraft") state.drafts = state.drafts.filter((draft) => draft.id !== action.draftId);
      return structuredClone(state);
    });
  });

  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  it("commits draft deletion through the domain command seam and advances the focus", async () => {
    render(WorkspacePage, { props: { context: "drafts" } });
    await screen.findByText(/conflict resolution happens on the leader/i);

    await fireEvent.click(screen.getByRole("button", { name: "Удалить" }));

    expect(execute).toHaveBeenCalledWith({ kind: "discardDraft", draftId: "draft-001" });
    await waitFor(() => expect(screen.getByText(/selectively simplified/i)).toBeTruthy());
  });

  it("keeps an opening failure local to the recoverable library screen", async () => {
    commandFactory.mockRejectedValueOnce(new Error("Хранилище недоступно"));

    render(WorkspacePage, { props: { context: "dashboard" } });

    expect((await screen.findByRole("alert")).textContent).toContain("Хранилище недоступно");
    expect(screen.getByRole("button", { name: "Повторить открытие" })).toBeTruthy();
  });
});
