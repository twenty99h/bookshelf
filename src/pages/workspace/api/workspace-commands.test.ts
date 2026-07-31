import { beforeEach, describe, expect, it, vi } from "vitest";
import { createWorkspaceCommands } from "./workspace-commands";
import { activeLibraryFixture } from "../model/workspace-fixtures";

const native = vi.hoisted(() => ({
  open: vi.fn(),
  convertFileSrc: vi.fn(),
  importPdfFromDialog: vi.fn(),
  bookFilePath: vi.fn(),
  loadLibrary: vi.fn(),
  executeLibraryAction: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: native.open, save: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ convertFileSrc: native.convertFileSrc }));
vi.mock("@/shared/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@/shared/api")>()),
  importPdfFromDialog: native.importPdfFromDialog,
  bookFilePath: native.bookFilePath,
  loadLibrary: native.loadLibrary,
  executeLibraryAction: native.executeLibraryAction,
}));

describe("native workspace command seam", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal("__BOOKSHELF_BROWSER_ADAPTER__", false);
  });

  it("keeps one native import, Reader URL, position save, and restart restoration chain", async () => {
    const restored = activeLibraryFixture();
    const restoredBook = restored.books.find((book) => book.id === "book-distributed")!;
    restoredBook.reading = { page: 2, zoom: 1.35, scroll: 0.64 };
    native.importPdfFromDialog.mockResolvedValue({ state: restored, bookId: "book-distributed", duplicate: false });
    native.bookFilePath.mockResolvedValue("C:\\AppData\\Bookshelf\\books\\book-native.pdf");
    native.convertFileSrc.mockReturnValue("http://asset.localhost/books/book-native.pdf");
    native.executeLibraryAction.mockResolvedValue(restored);
    native.loadLibrary.mockResolvedValue(restored);

    const commands = await createWorkspaceCommands();
    const imported = await commands.importPdf();
    const readerUrl = await commands.bookUrl("book-distributed");
    await commands.execute({
      kind: "updateReading",
      bookId: "book-distributed",
      page: 2,
      zoom: 1.35,
      scroll: 0.64,
    });
    const restarted = await (await createWorkspaceCommands()).load();

    expect(native.importPdfFromDialog).toHaveBeenCalledOnce();
    expect(native.bookFilePath).toHaveBeenCalledWith("book-distributed");
    expect(native.convertFileSrc).toHaveBeenCalledWith("C:\\AppData\\Bookshelf\\books\\book-native.pdf");
    expect(imported?.bookId).toBe("book-distributed");
    expect(readerUrl).toBe("http://asset.localhost/books/book-native.pdf");
    expect(restarted.books.find((book) => book.id === "book-distributed")?.reading).toEqual({
      page: 2,
      zoom: 1.35,
      scroll: 0.64,
    });
  });
});
