import { beforeEach, describe, expect, it, vi } from "vitest";
import { createWorkspaceCommands } from "./workspace-commands";

const native = vi.hoisted(() => ({
  open: vi.fn(),
  convertFileSrc: vi.fn(),
  importPdf: vi.fn(),
  bookFilePath: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: native.open, save: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ convertFileSrc: native.convertFileSrc }));
vi.mock("@/shared/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@/shared/api")>()),
  importPdf: native.importPdf,
  bookFilePath: native.bookFilePath,
}));

describe("native workspace command seam", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal("__BOOKSHELF_BROWSER_ADAPTER__", false);
  });

  it("imports through the system dialog and converts the stored local PDF path for Reader", async () => {
    native.open.mockResolvedValue("C:\\Books\\distributed.pdf");
    native.importPdf.mockResolvedValue({ state: { books: [] }, bookId: "book-native", duplicate: false });
    native.bookFilePath.mockResolvedValue("C:\\AppData\\Bookshelf\\books\\book-native.pdf");
    native.convertFileSrc.mockReturnValue("http://asset.localhost/books/book-native.pdf");

    const commands = await createWorkspaceCommands();
    const imported = await commands.importPdf();
    const readerUrl = await commands.bookUrl("book-native");

    expect(native.open).toHaveBeenCalledWith({ multiple: false, filters: [{ name: "PDF", extensions: ["pdf"] }] });
    expect(native.importPdf).toHaveBeenCalledWith("C:\\Books\\distributed.pdf");
    expect(native.bookFilePath).toHaveBeenCalledWith("book-native");
    expect(native.convertFileSrc).toHaveBeenCalledWith("C:\\AppData\\Bookshelf\\books\\book-native.pdf");
    expect(imported?.bookId).toBe("book-native");
    expect(readerUrl).toBe("http://asset.localhost/books/book-native.pdf");
  });
});
