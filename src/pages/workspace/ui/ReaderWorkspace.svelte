<script lang="ts">
  import { onMount, tick } from "svelte";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { type Book, type SourceFragment } from "@/shared/api";
  import { useWorkspaceSession } from "../model/workspace-session.svelte";
  import ReaderView from "./ReaderView.svelte";

  let { selectedBook }: { selectedBook: Book | null } = $props();

  const session = useWorkspaceSession();
  const library = $derived(session.library);
  const commands = $derived(session.commands);

  let sidebar = $state<"note" | "outline" | "search" | null>(null);
  let sidebarWidth = $state(400);
  let zoom = $state(1.15);
  let page = $state(286);
  let mode = $state<"muted" | "original" | "dark">("muted");
  let images = $state(true);
  let search = $state("");
  let searchResults = $state<{ page: number; excerpt: string }[]>([]);
  let saveState = $state<"saved" | "saving" | "error">("saved");
  let sidebarTrigger = $state<HTMLButtonElement | null>(null);
  let documentUrl = $state<string | null>(null);

  onMount(async () => {
    await session.load();
    await tick();
    if (!selectedBook || !session.commands) return;
    const query = new URLSearchParams(location.search);
    page = Number(query.get("sourcePage")) || selectedBook.reading.page;
    zoom = selectedBook.reading.zoom;
    mode =
      selectedBook.reader.documentMode === "mutedLight"
        ? "muted"
        : selectedBook.reader.documentMode === "darkInverted"
          ? "dark"
          : "original";
    images = selectedBook.reader.invertImages;
    sidebarWidth = selectedBook.reader.sidebarWidth;
    sidebar = selectedBook.reader.sidebarOpen ? selectedBook.reader.sidebarTab : null;
    documentUrl = await session.commands.bookUrl(selectedBook.id);
  });

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && sidebar) closeSidebar();
    if (event.ctrlKey && event.key.toLowerCase() === "f") {
      event.preventDefault();
      setSidebar("search");
    }
  }

  function setSidebar(tab: "note" | "outline" | "search") {
    sidebar = tab;
    void persistPreferences();
  }

  function closeSidebar() {
    sidebar = null;
    void persistPreferences();
    queueMicrotask(() => sidebarTrigger?.focus());
  }

  function rememberSidebarTrigger(node: HTMLButtonElement) {
    sidebarTrigger = node;
    return () => {
      if (sidebarTrigger === node) sidebarTrigger = null;
    };
  }

  async function persistPreferences() {
    if (!selectedBook || !commands) return;
    await session.execute(
      {
        kind: "updateReaderPreferences",
        bookId: selectedBook.id,
        preferences: {
          documentMode: mode === "muted" ? "mutedLight" : mode === "dark" ? "darkInverted" : "original",
          invertImages: images,
          sidebarOpen: sidebar !== null,
          sidebarTab: sidebar ?? "note",
          sidebarWidth,
        },
      },
      "",
    );
  }

  async function saveDraft(excerpt: string, comment: string, fragments: SourceFragment[]): Promise<string | null> {
    if (!selectedBook || selectedBook.studyStatus === "completed" || !excerpt.trim()) return null;
    const existingDraftIds = new Set(library?.drafts.map((draft) => draft.id) ?? []);
    const saved = await session.execute(
      fragments.length
        ? {
            kind: "captureDraftSources",
            bookId: selectedBook.id,
            section: "Глава 5 · Репликация",
            fragments: fragments.map((fragment) => ({ ...fragment })),
            comment,
          }
        : {
            kind: "captureDraft",
            bookId: selectedBook.id,
            section: "Глава 5 · Репликация",
            page,
            excerpt,
            context: "Фрагмент сохранён из непрерывного режима чтения.",
            comment,
          },
      "Черновая заметка сохранена",
    );
    if (!saved) return null;
    return library?.drafts.find((draft) => !existingDraftIds.has(draft.id))?.id ?? null;
  }

  async function createIdea(draftId: string, formulation: string): Promise<boolean> {
    if (!draftId || !formulation.trim()) return false;
    const draft = library?.drafts.find((item) => item.id === draftId);
    if (!draft) return false;
    return session.execute(
      {
        kind: "resolveDraftAsIdea",
        draftId: draft.id,
        formulation,
        section: draft.section,
        assignments: [],
      },
      "Идея сохранена; назначение идеи можно выбрать позже",
    );
  }

  async function savePosition(nextPage = page) {
    if (!selectedBook) return;
    page = nextPage;
    saveState = "saving";
    saveState = (await session.execute(
      { kind: "updateReading", bookId: selectedBook.id, page: nextPage, zoom, scroll: 0.32 },
      "",
    ))
      ? "saved"
      : "error";
  }

  function changeZoom(delta: number) {
    zoom = Math.min(2, Math.max(0.5, zoom + delta));
    void savePosition();
  }

  function openSavedSource(draftId: string, source: SourceFragment) {
    void savePosition(source.page).then(() => goto(resolve(`/drafts?draft=${encodeURIComponent(draftId)}`)));
  }

  function savePdfPosition(nextPage: number, scroll: number) {
    if (!selectedBook) return;
    page = nextPage;
    saveState = "saving";
    void session
      .execute({ kind: "updateReading", bookId: selectedBook.id, page: nextPage, zoom, scroll }, "")
      .then((saved) => (saveState = saved ? "saved" : "error"));
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<ReaderView
  {library}
  {selectedBook}
  {documentUrl}
  bind:page
  bind:zoom
  bind:mode
  bind:images
  bind:sidebar
  bind:sidebarWidth
  bind:search
  bind:searchResults
  {saveState}
  onChangeZoom={changeZoom}
  onSetSidebar={setSidebar}
  onCloseSidebar={closeSidebar}
  {rememberSidebarTrigger}
  onSavePosition={savePosition}
  onPdfPosition={savePdfPosition}
  onSourceSelect={openSavedSource}
  onSearchResults={(results) => (searchResults = results)}
  onSaveOutline={(outline) => {
    if (selectedBook && outline.length && selectedBook.outline.length === 0)
      void session.execute({ kind: "saveOutline", bookId: selectedBook.id, outline }, "");
  }}
  onSaveDraft={saveDraft}
  onCreateIdea={createIdea}
  onPersistPreferences={persistPreferences}
/>
