import { beforeEach, describe, expect, it } from "vitest";
import { browserWorkspaceCommands } from "./browser-workspace-commands";

describe("browser workspace command seam", () => {
  beforeEach(() => window.__BOOKSHELF_TEST__?.reset("active"));

  it("atomically pauses the prior active study", async () => {
    const state = await browserWorkspaceCommands.execute({ kind: "activateStudy", bookId: "book-domain" });

    expect(state.activeStudyBookId).toBe("book-domain");
    expect(state.books.find((book) => book.id === "book-distributed")?.studyStatus).toBe("paused");
    expect(state.books.find((book) => book.id === "book-domain")?.studyStatus).toBe("active");
  });

  it("preserves every source fragment when a draft becomes an idea", async () => {
    const initial = await browserWorkspaceCommands.load();
    const draft = initial.drafts.find((item) => item.fragments.length === 2);
    expect(draft).toBeDefined();

    const state = await browserWorkspaceCommands.execute({
      kind: "resolveDraftAsIdea",
      draftId: draft!.id,
      formulation: "Транзакция скрывает несколько разных классов конкурентных отказов.",
      section: draft!.section,
      assignments: ["recall"],
    });

    expect(state.ideas[0]?.fragments).toEqual(draft!.fragments);
    expect(state.drafts).not.toContainEqual(expect.objectContaining({ id: draft!.id }));
  });

  it("keeps the farthest page when the reader returns to an earlier source", async () => {
    await browserWorkspaceCommands.execute({
      kind: "updateReading",
      bookId: "book-distributed",
      page: 330,
      zoom: 1.2,
      scroll: 0.4,
    });
    const state = await browserWorkspaceCommands.execute({
      kind: "updateReading",
      bookId: "book-distributed",
      page: 120,
      zoom: 1.2,
      scroll: 0.1,
    });

    expect(state.books.find((book) => book.id === "book-distributed")?.farthestPage).toBe(330);
  });
});
