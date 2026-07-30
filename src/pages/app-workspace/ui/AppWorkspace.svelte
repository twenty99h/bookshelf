<script module lang="ts">
  function assignmentLabel(assignment: string): string {
    return (
      { recall: "Восстановление", transfer: "Передача", experiment: "Практика", mastered: "Освоено" }[assignment] ??
      assignment
    );
  }
</script>

<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import {
    ArrowLeft,
    BookOpen,
    BookCopy,
    Brain,
    Check,
    ChevronRight,
    Command,
    FileArchive,
    FlaskConical,
    Gauge,
    Library,
    ListTree,
    Menu,
    MoreHorizontal,
    PanelRight,
    Plus,
    Search,
    Settings,
    Sparkles,
    StickyNote,
    X,
    ZoomIn,
    ZoomOut,
  } from "@lucide/svelte";
  import { Button, CheckboxField, DialogModal, SelectField, TextArea, TextField } from "@/shared/ui";
  import { commandErrorMessage, type Book, type Idea, type LibraryAction, type LibraryState } from "@/shared/api";
  import { createWorkspaceCommands, type WorkspaceCommands } from "../api/workspace-commands";
  import type { WorkspaceContext } from "../model/workspace-context";
  import ContinuousPdfReader from "./ContinuousPdfReader.svelte";

  let {
    context,
    resourceId,
  }: {
    context: WorkspaceContext;
    resourceId?: string;
  } = $props();

  let commands = $state<WorkspaceCommands | null>(null);
  let library = $state.raw<LibraryState | null>(null);
  let loading = $state(true);
  let busy = $state(false);
  let error = $state("");
  let feedback = $state("");
  let paletteOpen = $state(false);
  let paletteQuery = $state("");
  let paletteResults = $state<{ id: string; kind: string; title: string; context: string }[]>([]);

  let libraryFilter = $state("all");
  let librarySort = $state("recent");
  let draftMode = $state<"focus" | "list">("focus");
  let draftFormulation = $state("");
  let selectedTopic = $state("all");
  let recallAnswer = $state("");
  let recallRevealed = $state(false);
  let experimentStep = $state("running");
  let completionStep = $state(1);
  let retrospective = $state("");
  let significantIdeas = $state<string[]>([]);

  let readerSidebar = $state<"note" | "outline" | "search" | null>(null);
  let readerSidebarWidth = $state(400);
  let readerZoom = $state(1.15);
  let readerPage = $state(286);
  let readerMode = $state("muted");
  let readerImages = $state(true);
  let readerSearch = $state("");
  let readerExcerpt = $state("");
  let readerFragments = $state<{ page: number; excerpt: string; context: string }[]>([]);
  let readerComment = $state("");
  let saveState = $state<"saved" | "saving" | "error">("saved");
  let sidebarTrigger = $state<HTMLButtonElement | null>(null);
  let readerDocumentUrl = $state<string | null>(null);

  const settingsNavigation = [
    { icon: Menu, label: "Интерфейс" },
    { icon: Library, label: "Библиотека" },
    { icon: FileArchive, label: "Резервные копии" },
    { icon: Sparkles, label: "ИИ" },
  ];
  const readerTabs: { id: "note" | "outline" | "search"; label: string; icon: typeof StickyNote }[] = [
    { id: "note", label: "Заметка", icon: StickyNote },
    { id: "outline", label: "Оглавление", icon: ListTree },
    { id: "search", label: "Поиск", icon: Search },
  ];

  const activeBook = $derived(library?.books.find((book) => book.id === library?.activeStudyBookId) ?? null);
  const selectedBook = $derived(
    library?.books.find((book) => book.id === resourceId) ?? activeBook ?? library?.books[0] ?? null,
  );
  const selectedIdea = $derived(library?.ideas.find((idea) => idea.id === resourceId) ?? library?.ideas[0] ?? null);
  const focusedDraft = $derived(library?.drafts[0] ?? null);
  const unfinishedCount = $derived(
    (library?.drafts.length ?? 0) +
      (library?.experiments.filter((experiment) => !experiment.completed).length ?? 0) +
      (library?.reviews.filter((review) => review.pending).length ?? 0),
  );
  const filteredBooks = $derived.by(() => {
    if (!library) return [];
    const snapshot = library;
    const books = snapshot.books.filter((book) => {
      if (libraryFilter === "all") return true;
      if (libraryFilter === "active") return book.id === snapshot.activeStudyBookId;
      if (libraryFilter === "completed") return book.studyCompleted;
      if (libraryFilter === "ready") return book.readingCompleted && !book.studyCompleted;
      if (libraryFilter === "paused") return book.id !== snapshot.activeStudyBookId && !book.studyCompleted;
      return true;
    });
    return books.toSorted((a, b) => {
      if (librarySort === "title") return a.title.localeCompare(b.title, "ru");
      if (librarySort === "progress") return b.reading.page - a.reading.page;
      return snapshot.books.indexOf(a) - snapshot.books.indexOf(b);
    });
  });

  onMount(async () => {
    try {
      commands = await createWorkspaceCommands();
      library = await commands.load();
      if (selectedBook) {
        readerPage = selectedBook.reading.page;
        readerZoom = selectedBook.reading.zoom;
        readerMode =
          selectedBook.reader.documentMode === "mutedLight"
            ? "muted"
            : selectedBook.reader.documentMode === "darkInverted"
              ? "dark"
              : "original";
        readerImages = selectedBook.reader.invertImages;
        readerSidebarWidth = selectedBook.reader.sidebarWidth;
        readerSidebar = selectedBook.reader.sidebarOpen ? selectedBook.reader.sidebarTab : null;
        if (context === "reader") readerDocumentUrl = await commands.bookUrl(selectedBook.id);
      }
    } catch (cause) {
      error = commandErrorMessage(cause);
    } finally {
      loading = false;
    }
  });

  async function run(action: LibraryAction, message = "Сохранено") {
    if (!commands) return;
    busy = true;
    error = "";
    feedback = "";
    try {
      library = await commands.execute(action);
      feedback = message;
    } catch (cause) {
      error = commandErrorMessage(cause);
    } finally {
      busy = false;
    }
  }

  async function importBook() {
    if (!commands) return;
    busy = true;
    try {
      const snapshot = await commands.importPdf();
      if (snapshot) {
        library = snapshot;
        const imported = snapshot.books[0];
        if (imported) await goto(resolve("/library/[bookId]", { bookId: imported.id }));
      }
    } catch (cause) {
      error = commandErrorMessage(cause);
    } finally {
      busy = false;
    }
  }

  async function searchPalette() {
    if (!commands) return;
    paletteResults = await commands.search(paletteQuery);
  }

  function openPaletteResult(result: { id: string; kind: string }) {
    paletteOpen = false;
    if (result.kind === "book") goto(resolve("/library/[bookId]", { bookId: result.id }));
    else if (result.kind === "idea") goto(resolve("/knowledge/[ideaId]", { ideaId: result.id }));
    else if (result.kind === "draft" || result.kind === "material") goto(resolve("/drafts"));
  }

  function handleGlobalKeydown(event: KeyboardEvent) {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k" && context !== "reader") {
      event.preventDefault();
      paletteOpen = true;
    }
    if (context === "reader" && event.key === "Escape" && readerSidebar) {
      closeReaderSidebar();
    }
    if (context === "reader" && event.ctrlKey && event.key === "Enter" && readerSidebar === "note") {
      event.preventDefault();
      saveReaderDraft();
    }
    if (context === "reader" && event.ctrlKey && event.key.toLowerCase() === "f") {
      event.preventDefault();
      setReaderSidebar("search");
    }
  }

  function captureSelection() {
    const selection = window.getSelection()?.toString().trim();
    if (!selection) return;
    readerExcerpt = selection;
    setReaderSidebar("note");
  }

  function setReaderSidebar(tab: "note" | "outline" | "search") {
    readerSidebar = tab;
    void persistReaderPreferences();
  }

  function closeReaderSidebar() {
    readerSidebar = null;
    void persistReaderPreferences();
    queueMicrotask(() => sidebarTrigger?.focus());
  }

  function rememberSidebarTrigger(node: HTMLButtonElement) {
    sidebarTrigger = node;
    return () => {
      if (sidebarTrigger === node) sidebarTrigger = null;
    };
  }

  async function persistReaderPreferences() {
    if (!selectedBook || !commands) return;
    await run(
      {
        kind: "updateReaderPreferences",
        bookId: selectedBook.id,
        preferences: {
          documentMode: readerMode === "muted" ? "mutedLight" : readerMode === "dark" ? "darkInverted" : "original",
          invertImages: readerImages,
          sidebarOpen: readerSidebar !== null,
          sidebarTab: readerSidebar ?? "note",
          sidebarWidth: readerSidebarWidth,
        },
      },
      "",
    );
  }

  async function saveReaderDraft() {
    if (!selectedBook || !readerExcerpt.trim()) return;
    await run(
      readerFragments.length
        ? {
            kind: "captureDraftSources",
            bookId: selectedBook.id,
            section: "Глава 5 · Репликация",
            fragments: readerFragments,
            comment: readerComment,
          }
        : {
            kind: "captureDraft",
            bookId: selectedBook.id,
            section: "Глава 5 · Репликация",
            page: readerPage,
            excerpt: readerExcerpt,
            context: "Фрагмент сохранён из непрерывного режима чтения.",
            comment: readerComment,
          },
      "Черновая заметка сохранена",
    );
    readerExcerpt = "";
    readerFragments = [];
    readerComment = "";
  }

  async function saveReaderPosition(page = readerPage) {
    if (!selectedBook) return;
    readerPage = page;
    saveState = "saving";
    try {
      await run({ kind: "updateReading", bookId: selectedBook.id, page, zoom: readerZoom, scroll: 0.32 }, "");
      saveState = "saved";
    } catch {
      saveState = "error";
    }
  }

  function capturePdfSelection(fragments: { page: number; excerpt: string; context: string }[]) {
    const first = fragments[0];
    if (!first) return;
    readerPage = first.page;
    readerFragments = fragments;
    readerExcerpt = fragments.map((fragment) => fragment.excerpt).join("\n");
    setReaderSidebar("note");
  }

  function savePdfPosition(page: number, scroll: number) {
    readerPage = page;
    saveState = "saving";
    void run({ kind: "updateReading", bookId: selectedBook!.id, page, zoom: readerZoom, scroll }, "")
      .then(() => (saveState = "saved"))
      .catch(() => (saveState = "error"));
  }

  async function resolveDraft() {
    if (!focusedDraft || !draftFormulation.trim()) return;
    await run(
      {
        kind: "resolveDraftAsIdea",
        draftId: focusedDraft.id,
        formulation: draftFormulation,
        section: focusedDraft.section,
        assignments: ["recall"],
      },
      "Идея сформулирована; источник сохранён",
    );
    draftFormulation = "";
  }

  function bookStatus(book: Book): string {
    if (book.studyCompleted) return "Завершено";
    if (book.id === library?.activeStudyBookId) return "Активное изучение";
    if (book.readingCompleted) return "Готово к завершению";
    return "Приостановлено";
  }

  function bookForIdea(idea: Idea): Book | undefined {
    return library?.books.find((book) => book.id === idea.bookId);
  }

  function contextTitle(): string {
    const titles: Record<WorkspaceContext, string> = {
      dashboard: "Рабочий стол",
      library: "Личная библиотека",
      book: selectedBook?.title ?? "Книга",
      reader: selectedBook?.title ?? "Режим чтения",
      drafts: "Разбор черновиков",
      knowledge: "Знания",
      idea: "Идея книги",
      practice: "Практика",
      completion: "Завершение изучения",
      settings: "Настройки",
    };
    return titles[context];
  }

  function toggleSignificantIdea(ideaId: string, checked: boolean) {
    significantIdeas = checked
      ? [...significantIdeas, ideaId]
      : significantIdeas.filter((candidate) => candidate !== ideaId);
  }
</script>

<svelte:window onkeydown={handleGlobalKeydown} />
<svelte:document onselectionchange={captureSelection} />

{#if context === "reader"}
  {@render readerView()}
{:else}
  <div class="min-h-screen bg-night text-mist">
    <div class="grid min-h-screen grid-cols-[248px_minmax(0,1fr)] max-[1100px]:grid-cols-[210px_minmax(0,1fr)]">
      <aside
        class="sticky top-0 flex h-screen flex-col border-r border-white/8 bg-night px-4 py-5"
        aria-label="Основная навигация"
      >
        <a href={resolve("/")} class="mb-8 flex items-center gap-3 rounded-lg px-3 py-2 text-mist no-underline">
          <span class="grid size-9 place-items-center rounded-md border border-amber/35 bg-amber/10 text-amber"
            ><BookCopy class="size-5" /></span
          >
          <span><b class="block tracking-wide">Bookshelf</b><small class="text-mist-dim">Личное изучение</small></span>
        </a>
        <nav class="grid gap-1">
          {@render navItem("dashboard", "/", "Рабочий стол", Gauge)}
          {@render navItem("library", "/library", "Библиотека", Library)}
          {@render navItem("drafts", "/drafts", "Черновики", StickyNote, library?.drafts.length)}
          {@render navItem("knowledge", "/knowledge", "Знания", Brain)}
          {@render navItem("practice", "/practice", "Практика", FlaskConical)}
        </nav>
        <div class="mt-auto grid gap-2">
          <button
            class="flex min-h-10 items-center gap-3 rounded-md px-3 text-left text-sm text-mist-dim hover:bg-slate hover:text-mist focus-visible:outline-2 focus-visible:outline-iris"
            onclick={() => (paletteOpen = true)}
          >
            <Command class="size-4" /><span>Быстрый переход</span><kbd class="ml-auto font-mono text-[11px]">Ctrl K</kbd
            >
          </button>
          {@render navItem("settings", "/settings", "Настройки", Settings)}
        </div>
      </aside>

      <main class="min-w-0 bg-graphite">
        <header class="flex min-h-20 items-center justify-between border-b border-white/8 px-8 max-[1280px]:px-6">
          <div>
            <p class="mb-1 font-mono text-[11px] uppercase tracking-[0.16em] text-mist-dim">Bookshelf / {context}</p>
            <h1 class="text-xl font-semibold tracking-tight">{contextTitle()}</h1>
          </div>
          <div class="flex items-center gap-3">
            {#if feedback}<span role="status" class="font-mono text-xs text-success">{feedback}</span>{/if}
            <span class="rounded-md border border-white/10 bg-slate px-3 py-2 font-mono text-xs text-mist-dim"
              >Локальная библиотека</span
            >
          </div>
        </header>

        <div class="mx-auto max-w-[1500px] p-8 max-[1280px]:p-6">
          {#if loading}
            <div class="grid min-h-[60vh] place-items-center" role="status">Открываем личную библиотеку…</div>
          {:else if error && !library}
            <section
              class="mx-auto mt-24 max-w-xl rounded-xl border border-danger/40 bg-slate p-8 text-center"
              role="alert"
            >
              <h2 class="text-xl font-semibold">Личная библиотека не открылась</h2>
              <p class="mt-3 text-mist-dim">{error}</p>
              <div class="mt-6"><Button onclick={() => location.reload()}>Повторить открытие</Button></div>
            </section>
          {:else if library}
            {#if error}<p class="mb-4 rounded-lg border border-danger/40 bg-danger/10 p-3 text-sm" role="alert">
                {error}
              </p>{/if}
            {#if context === "dashboard"}{@render dashboardView()}
            {:else if context === "library"}{@render libraryView()}
            {:else if context === "book"}{@render bookView()}
            {:else if context === "drafts"}{@render draftsView()}
            {:else if context === "knowledge" || context === "idea"}{@render knowledgeView()}
            {:else if context === "practice"}{@render practiceView()}
            {:else if context === "completion"}{@render completionView()}
            {:else if context === "settings"}{@render settingsView()}{/if}
          {/if}
        </div>
      </main>
    </div>
  </div>

  <DialogModal
    bind:open={paletteOpen}
    title="Быстрый переход"
    description="Найдите книгу, идею, тему, черновик или материал."
  >
    {#snippet trigger()}<span class="sr-only">Открыть быстрый переход</span>{/snippet}
    <form
      class="grid gap-3"
      onsubmit={(event) => {
        event.preventDefault();
        searchPalette();
      }}
    >
      <TextField id="command-search" label="Поиск" bind:value={paletteQuery} placeholder="Название или формулировка" />
      <Button type="submit">Найти</Button>
    </form>
    <div class="grid gap-1" aria-live="polite">
      {#if paletteQuery && paletteResults.length === 0}<p class="text-sm text-mist-dim">
          Совпадений нет. Измените запрос, введённый текст сохранён.
        </p>{/if}
      {#each paletteResults as result (`${result.kind}-${result.id}`)}
        <button
          class="flex items-center gap-3 rounded-lg border border-white/8 bg-slate p-3 text-left hover:border-iris/50"
          onclick={() => openPaletteResult(result)}
        >
          <Search class="size-4 text-iris" /><span
            ><b class="line-clamp-1">{result.title}</b><small class="block text-mist-dim">{result.context}</small></span
          >
        </button>
      {/each}
    </div>
  </DialogModal>
{/if}

{#snippet navItem(
  itemContext: WorkspaceContext,
  href: "/" | "/library" | "/drafts" | "/knowledge" | "/practice" | "/settings",
  label: string,
  Icon: typeof Gauge,
  badge?: number,
)}
  <a
    href={resolve(href)}
    aria-current={context === itemContext || (itemContext === "knowledge" && context === "idea") ? "page" : undefined}
    class="group flex min-h-11 items-center gap-3 rounded-md border border-transparent px-3 text-sm text-mist-dim no-underline hover:bg-slate hover:text-mist aria-[current=page]:border-iris/20 aria-[current=page]:bg-iris/12 aria-[current=page]:text-mist"
  >
    <Icon class="size-[18px] group-aria-[current=page]:text-iris" /><span>{label}</span>
    {#if badge}<span class="ml-auto rounded-full bg-amber/15 px-2 py-0.5 font-mono text-[11px] text-amber">{badge}</span
      >{/if}
  </a>
{/snippet}

{#snippet dashboardView()}
  {#if library!.books.length === 0}
    <section class="grid min-h-[72vh] place-items-center">
      <div class="max-w-xl text-center">
        <div
          class="mx-auto mb-8 grid h-32 w-24 place-items-center rounded-r-xl border border-amber/30 bg-slate shadow-[inset_7px_0_0_#d6a24a]"
        >
          <BookOpen class="size-10 text-amber" />
        </div>
        <p class="mb-3 font-mono text-xs uppercase tracking-[0.18em] text-amber">
          Личная библиотека · только на этом компьютере
        </p>
        <h2 class="text-4xl font-semibold tracking-[-0.03em]">Начните с одной книги, к которой хотите возвращаться</h2>
        <p class="mx-auto mt-5 max-w-lg text-base leading-7 text-mist-dim">
          Bookshelf сохраняет PDF, место чтения и ваши формулировки локально. Облачной синхронизации и обязательной
          настройки нет.
        </p>
        <div class="mt-8">
          <Button variant="primary" disabled={busy} onclick={importBook}
            ><Plus class="mr-2 size-4" />Импортировать PDF</Button
          >
        </div>
      </div>
    </section>
  {:else}
    <section class="grid grid-cols-[minmax(0,1.55fr)_minmax(330px,.75fr)] gap-6 max-[1280px]:grid-cols-1">
      <div class="grid gap-6">
        <article class="relative overflow-hidden rounded-xl border border-white/8 bg-slate p-8">
          <div class="absolute inset-y-0 left-0 w-1 bg-iris"></div>
          <p class="font-mono text-xs uppercase tracking-[0.15em] text-iris">Активное изучение</p>
          <h2 class="mt-4 max-w-3xl text-4xl font-semibold tracking-[-0.035em]">{activeBook?.title}</h2>
          <p class="mt-3 text-mist-dim">
            Глава 5 · Репликация · последняя позиция <span class="font-mono text-mist">{activeBook?.reading.page}</span>
          </p>
          <div class="mt-9 flex items-center gap-3">
            <a
              class="inline-flex min-h-11 items-center gap-2 rounded-md bg-iris-strong px-5 font-semibold text-white no-underline hover:bg-[#4d48aa] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-iris"
              href={resolve("/reader/[bookId]", { bookId: activeBook?.id ?? "book" })}
              ><BookOpen class="size-4" />Продолжить чтение</a
            >
            <a
              class="inline-flex min-h-11 items-center rounded-md border border-white/12 px-4 text-sm text-mist no-underline hover:bg-white/5"
              href={resolve("/library/[bookId]", { bookId: activeBook?.id ?? "book" })}>Открыть книгу</a
            >
          </div>
          <div class="mt-8 grid grid-cols-3 gap-6 border-t border-white/8 pt-6">
            {@render metric("Последняя позиция", `${activeBook?.reading.page}`, "страница")}
            {@render metric("Пройдено по тексту", `${Math.max(activeBook?.reading.page ?? 0, 312)}`, "дальняя позиция")}
            {@render metric(
              "Черновики книги",
              `${library!.drafts.filter((draft) => draft.bookId === activeBook?.id).length}`,
              "на разбор",
            )}
          </div>
        </article>

        <article class="rounded-xl border border-white/8 bg-slate p-6">
          <div class="flex items-end justify-between gap-4">
            <div>
              <p class="font-mono text-xs uppercase tracking-[0.15em] text-mist-dim">Последние семь дней</p>
              <h2 class="mt-2 text-xl font-semibold">Текст переходит в знание</h2>
            </div>
            <span class="font-mono text-xs text-mist-dim">24–30 июля</span>
          </div>
          <div class="mt-6 grid grid-cols-4 gap-px overflow-hidden rounded-lg border border-white/8 bg-white/8">
            {@render weeklyCell("Текст", "+47 стр.", "Самая дальняя позиция")}
            {@render weeklyCell("Идеи", "3", "Сформулировано")}
            {@render weeklyCell("Восстановления", "2", "Решения читателя")}
            {@render weeklyCell("Практика", "1", "Продвижение")}
          </div>
        </article>
      </div>
      <aside class="grid content-start gap-6">
        <article class="rounded-xl border border-white/8 bg-slate p-6">
          <div class="flex items-center justify-between">
            <div>
              <p class="font-mono text-xs uppercase tracking-[0.15em] text-mist-dim">Незавершённая работа</p>
              <h2 class="mt-2 text-xl font-semibold">{unfinishedCount} требуют решения</h2>
            </div>
            <span class="grid size-10 place-items-center rounded-full bg-amber/12 font-mono text-amber"
              >{unfinishedCount}</span
            >
          </div>
          <div class="mt-5 grid gap-1">
            {@render workRow("Черновые заметки", `${library!.drafts.length}`, "/drafts")}
            {@render workRow(
              "Проверки идей",
              `${library!.reviews.filter((review) => review.pending).length}`,
              "/knowledge",
            )}
            {@render workRow(
              "Эксперименты",
              `${library!.experiments.filter((experiment) => !experiment.completed).length}`,
              "/practice",
            )}
          </div>
          <p class="mt-5 text-sm leading-6 text-mist-dim">Эта работа сохранена и не мешает продолжить книгу.</p>
        </article>
        <article class="rounded-xl border border-white/8 bg-night/40 p-6">
          <p class="font-mono text-xs uppercase tracking-[0.15em] text-mist-dim">Рекомендуемый следующий шаг</p>
          <h3 class="mt-3 font-semibold">Разобрать источник о смене лидера</h3>
          <p class="mt-2 text-sm leading-6 text-mist-dim">
            Свежий фрагмент связан с текущей главой и займёт одно решение.
          </p>
          <a
            href={resolve("/drafts")}
            class="mt-4 inline-flex items-center gap-1 text-sm font-semibold text-iris no-underline"
            >Открыть черновик <ChevronRight class="size-4" /></a
          >
        </article>
      </aside>
    </section>
  {/if}
{/snippet}

{#snippet metric(label: string, value: string, detail: string)}<div>
    <span class="block font-mono text-2xl text-mist">{value}</span><span class="mt-1 block text-sm text-mist-dim"
      >{label}</span
    ><small class="font-mono text-[10px] uppercase text-mist-faint">{detail}</small>
  </div>{/snippet}
{#snippet weeklyCell(label: string, value: string, detail: string)}<div class="bg-night/30 p-4">
    <span class="font-mono text-xl">{value}</span><b class="mt-2 block text-sm">{label}</b><small class="text-mist-dim"
      >{detail}</small
    >
  </div>{/snippet}
{#snippet workRow(label: string, value: string, href: "/drafts" | "/knowledge" | "/practice")}<a
    href={resolve(href)}
    class="flex items-center rounded-md px-2 py-3 text-sm text-mist no-underline hover:bg-white/5"
    ><span>{label}</span><span class="ml-auto font-mono text-mist-dim">{value}</span><ChevronRight
      class="ml-2 size-4 text-mist-faint"
    /></a
  >{/snippet}

{#snippet libraryView()}
  <div class="mb-6 flex items-end justify-between gap-5">
    <div>
      <p class="max-w-2xl text-sm leading-6 text-mist-dim">
        Книги, позиции чтения и циклы изучения. Архив остаётся фильтром, не отдельным местом.
      </p>
    </div>
    <Button variant="primary" onclick={importBook}><Plus class="mr-2 size-4" />Импортировать PDF</Button>
  </div>
  <div class="mb-4 grid grid-cols-[minmax(0,1fr)_220px] gap-4">
    <div class="flex flex-wrap gap-2" aria-label="Фильтры библиотеки">
      {#each [["all", "Все"], ["active", "Активные"], ["paused", "Приостановленные"], ["ready", "Готовые"], ["completed", "Завершённые"]] as option (option[0])}<button
          class="rounded-md border px-3 py-2 text-sm data-[active=true]:border-iris/40 data-[active=true]:bg-iris/12 data-[active=true]:text-mist data-[active=false]:border-white/8 data-[active=false]:text-mist-dim"
          data-active={libraryFilter === option[0]}
          onclick={() => (libraryFilter = option[0] ?? "all")}>{option[1]}</button
        >{/each}
    </div>
    <SelectField
      label="Сортировка"
      value={librarySort}
      options={[
        { value: "recent", label: "Последнее обращение" },
        { value: "title", label: "Название" },
        { value: "progress", label: "Прогресс чтения" },
      ]}
      onValueChange={(value) => (librarySort = value)}
    />
  </div>
  <section class="overflow-hidden rounded-xl border border-white/8 bg-slate" aria-label="Книги">
    <div
      class="grid grid-cols-[54px_minmax(280px,1.5fr)_180px_130px_120px_150px] items-center gap-4 border-b border-white/8 px-5 py-3 font-mono text-[10px] uppercase tracking-[0.12em] text-mist-faint"
    >
      <span></span><span>Книга</span><span>Изучение</span><span>Позиция</span><span>Черновики</span><span></span>
    </div>
    {#each filteredBooks as book (book.id)}
      <article
        class="grid min-h-[76px] grid-cols-[54px_minmax(280px,1.5fr)_180px_130px_120px_150px] items-center gap-4 border-b border-white/6 px-5 last:border-0 hover:bg-white/[0.025]"
      >
        <div
          class="grid h-12 w-9 place-items-center rounded-r border border-white/10 bg-night/50 font-semibold text-amber shadow-[inset_3px_0_0_#d6a24a]"
        >
          {book.title[0]}
        </div>
        <div>
          <a
            href={resolve("/library/[bookId]", { bookId: book.id })}
            class="font-semibold text-mist no-underline hover:text-iris">{book.title}</a
          ><small class="mt-1 block text-mist-faint"
            >PDF · {book.hasTextLayer ? "текстовый слой" : "только изображение"}</small
          >
        </div>
        <span class="text-sm text-mist-dim">{bookStatus(book)}</span><span class="font-mono text-sm"
          >стр. {book.reading.page}</span
        ><span class="font-mono text-sm">{library!.drafts.filter((draft) => draft.bookId === book.id).length}</span>
        <a
          class="inline-flex min-h-9 items-center justify-center rounded-md bg-iris-strong px-3 text-sm font-semibold text-white no-underline"
          href={resolve("/reader/[bookId]", { bookId: book.id })}>{book.reading.page > 1 ? "Продолжить" : "Открыть"}</a
        >
      </article>
    {:else}<div class="p-10 text-center">
        <h2 class="font-semibold">В этом фильтре книг нет</h2>
        <button class="mt-3 text-sm text-iris" onclick={() => (libraryFilter = "all")}>Сбросить фильтр</button>
      </div>{/each}
  </section>
{/snippet}

{#snippet bookView()}
  {#if selectedBook}
    <div class="grid grid-cols-[180px_minmax(0,1fr)_260px] gap-8 max-[1280px]:grid-cols-[140px_minmax(0,1fr)]">
      <div
        class="grid h-60 place-items-center rounded-r-xl border border-white/10 bg-night/50 text-5xl font-semibold text-amber shadow-[inset_9px_0_0_#d6a24a]"
      >
        {selectedBook.title[0]}
      </div>
      <div>
        <p class="font-mono text-xs uppercase tracking-[.14em] text-iris">{bookStatus(selectedBook)}</p>
        <h2 class="mt-3 text-4xl font-semibold tracking-[-.035em]">{selectedBook.title}</h2>
        <p class="mt-4 text-mist-dim">
          PDF · 612 страниц · {selectedBook.hasTextLayer ? "есть оглавление и текстовый слой" : "без текстового слоя"}
        </p>
        <div class="mt-7 flex gap-3">
          <a
            class="inline-flex min-h-11 items-center gap-2 rounded-md bg-iris-strong px-5 font-semibold text-white no-underline"
            href={resolve("/reader/[bookId]", { bookId: selectedBook.id })}
            ><BookOpen class="size-4" />Продолжить чтение</a
          ><Button onclick={() => run({ kind: "activateStudy", bookId: selectedBook.id }, "Книга стала активной")}
            >Сделать активной</Button
          ><button
            aria-label="Другие действия"
            class="grid size-11 place-items-center rounded-md border border-white/10"><MoreHorizontal /></button
          >
        </div>
      </div>
      <aside class="rounded-xl border border-white/8 bg-slate p-5 max-[1280px]:col-span-2">
        <p class="font-mono text-xs uppercase text-mist-dim">Прогресс чтения</p>
        <div class="mt-4 flex items-end justify-between">
          <span class="font-mono text-3xl">{selectedBook.reading.page}</span><span class="text-sm text-mist-dim"
            >последняя</span
          >
        </div>
        <div class="mt-3 h-1.5 rounded-full bg-night"><div class="h-full w-[56%] rounded-full bg-iris"></div></div>
        <p class="mt-5 text-sm text-mist-dim">
          Самая дальняя позиция <span class="font-mono text-mist">312</span>. Возврат к источнику её не уменьшает.
        </p>
      </aside>
    </div>
    <nav class="mt-9 flex gap-7 border-b border-white/8" aria-label="Разделы книги">
      {#each ["Обзор", "Черновики", "Идеи", "Практика"] as tab (tab)}<button
          class="border-b-2 border-transparent px-1 pb-3 text-sm text-mist-dim first:border-iris first:text-mist"
          >{tab}</button
        >{/each}
    </nav>
    <section class="mt-6 grid grid-cols-3 gap-5">
      <article class="col-span-2 rounded-xl border border-white/8 bg-slate p-6">
        <h2 class="text-xl font-semibold">Продолжить с контекстом</h2>
        <p class="mt-2 text-mist-dim">Глава 5 · Репликация · страница {selectedBook.reading.page}</p>
        <div class="mt-6 border-l-2 border-amber pl-4">
          <small class="font-mono uppercase text-amber">Последний источник</small>
          <p class="mt-2 leading-6">
            {library!.drafts.find((draft) => draft.bookId === selectedBook.id)?.excerpt ??
              "У этой книги пока нет сохранённых источников."}
          </p>
        </div>
      </article>
      <article class="rounded-xl border border-white/8 bg-slate p-6">
        <p class="font-mono text-xs uppercase text-mist-dim">Прогресс изучения</p>
        <dl class="mt-4 grid gap-3 text-sm">
          <div class="flex">
            <dt>Разобрано</dt>
            <dd class="ml-auto font-mono">7</dd>
          </div>
          <div class="flex">
            <dt>Идей</dt>
            <dd class="ml-auto font-mono">{library!.ideas.filter((idea) => idea.bookId === selectedBook.id).length}</dd>
          </div>
          <div class="flex">
            <dt>Восстановлений</dt>
            <dd class="ml-auto font-mono">2</dd>
          </div>
          <div class="flex">
            <dt>Применений</dt>
            <dd class="ml-auto font-mono">1</dd>
          </div>
        </dl>
        <a
          href={resolve("/library/[bookId]/complete", { bookId: selectedBook.id })}
          class="mt-6 inline-flex text-sm font-semibold text-iris no-underline">Подвести итог изучения</a
        >
      </article>
    </section>
  {/if}
{/snippet}

{#snippet draftsView()}
  <div class="mb-6 flex items-center justify-between">
    <p class="max-w-2xl text-sm leading-6 text-mist-dim">
      Одно решение за раз. Полный список нужен только когда вы ищете конкретный материал.
    </p>
    <div class="flex rounded-md border border-white/8 p-1">
      <button
        class="rounded px-3 py-1.5 text-sm data-[active=true]:bg-iris/15"
        data-active={draftMode === "focus"}
        onclick={() => (draftMode = "focus")}>Разбор</button
      ><button
        class="rounded px-3 py-1.5 text-sm data-[active=true]:bg-iris/15"
        data-active={draftMode === "list"}
        onclick={() => (draftMode = "list")}>Все заметки</button
      >
    </div>
  </div>
  {#if !focusedDraft}<section
      class="grid min-h-[55vh] place-items-center rounded-xl border border-white/8 bg-slate text-center"
    >
      <div>
        <Check class="mx-auto size-9 text-success" />
        <h2 class="mt-4 text-2xl font-semibold">Всё разобрано</h2>
        <p class="mt-2 text-mist-dim">Можно вернуться к чтению или открыть знания.</p>
      </div>
    </section>
  {:else if draftMode === "list"}<section class="rounded-xl border border-white/8 bg-slate">
      {#each library!.drafts as draft (draft.id)}<article
          class="grid grid-cols-[180px_1fr_160px] gap-5 border-b border-white/8 p-5"
        >
          <span class="text-sm text-mist-dim"
            >{draft.section}<small class="block font-mono text-amber">стр. {draft.page}</small></span
          >
          <p>{draft.excerpt}</p>
          <button class="text-right text-sm font-semibold text-iris" onclick={() => (draftMode = "focus")}
            >Разобрать</button
          >
        </article>{/each}
    </section>
  {:else}<section class="grid grid-cols-[minmax(0,.9fr)_minmax(420px,1.1fr)] gap-6">
      <article class="rounded-xl border border-white/8 bg-night/35 p-7">
        <p class="font-mono text-xs uppercase tracking-[.14em] text-amber">
          Источник · {focusedDraft.section} · стр. {focusedDraft.page}
        </p>
        <blockquote class="mt-6 border-l-2 border-amber pl-5 text-lg leading-8">{focusedDraft.excerpt}</blockquote>
        <p class="mt-5 text-sm leading-6 text-mist-dim">{focusedDraft.context}</p>
        <a
          href={resolve("/reader/[bookId]", { bookId: focusedDraft.bookId })}
          class="mt-6 inline-flex text-sm text-amber no-underline">Открыть источник в книге</a
        >
      </article>
      <article class="rounded-xl border border-white/8 bg-slate p-7">
        <p class="font-mono text-xs uppercase tracking-[.14em] text-iris">
          Решение {library!.drafts.length > 1 ? `· ещё ${library!.drafts.length - 1}` : ""}
        </p>
        <h2 class="mt-3 text-2xl font-semibold">Сформулируйте самостоятельную идею</h2>
        <p class="mt-2 text-sm leading-6 text-mist-dim">Цитата останется источником, а не заменит вашу мысль.</p>
        <div class="mt-6">
          <TextArea
            id="draft-formulation"
            label="Моя формулировка"
            bind:value={draftFormulation}
            placeholder="Что это утверждение меняет в ваших решениях?"
          />
        </div>
        <div class="mt-5 flex flex-wrap gap-2">
          <Button variant="primary" onclick={resolveDraft} disabled={!draftFormulation.trim() || busy}
            >Создать идею</Button
          ><Button>Присоединить к идее</Button><Button>Отложить</Button><Button>Экспортировать</Button><Button
            >Удалить</Button
          >
        </div>
      </article>
    </section>{/if}
{/snippet}

{#snippet knowledgeView()}
  <div class="mb-5 flex items-end justify-between">
    <p class="max-w-xl text-sm leading-6 text-mist-dim">
      Авторские формулировки с источниками, назначениями и подтверждёнными связями.
    </p>
    <SelectField
      label="Тема знаний"
      value={selectedTopic}
      options={[
        { value: "all", label: "Все темы" },
        ...library!.topics.map((topic) => ({ value: topic.id, label: topic.name })),
      ]}
      onValueChange={(value) => (selectedTopic = value)}
    />
  </div>
  <section
    class="grid min-h-[680px] grid-cols-[38%_62%] overflow-hidden rounded-xl border border-white/8 bg-slate max-[1280px]:grid-cols-[42%_58%]"
  >
    <div class="border-r border-white/8">
      <div class="border-b border-white/8 p-4">
        <TextField id="knowledge-search" ariaLabel="Поиск по знаниям" placeholder="Найти формулировку" />
      </div>
      {#each library!.ideas.filter((idea) => selectedTopic === "all" || idea.topicIds.includes(selectedTopic)) as idea (idea.id)}<a
          href={resolve("/knowledge/[ideaId]", { ideaId: idea.id })}
          aria-current={selectedIdea?.id === idea.id ? "true" : undefined}
          class="block border-b border-white/8 p-5 text-mist no-underline hover:bg-white/[.025] aria-[current=true]:border-l-2 aria-[current=true]:border-l-iris aria-[current=true]:bg-iris/[.07]"
          ><small class="font-mono text-[10px] uppercase text-amber">{bookForIdea(idea)?.title} · {idea.section}</small>
          <h2 class="mt-2 line-clamp-3 text-[15px] font-semibold leading-6">{idea.formulation}</h2>
          <div class="mt-3 flex gap-2">
            {#each idea.assignments as assignment (assignment)}<span
                class="rounded bg-night/50 px-2 py-1 font-mono text-[10px] text-mist-dim"
                >{assignmentLabel(assignment)}</span
              >{/each}
          </div></a
        >{/each}
    </div>
    {#if selectedIdea}<article class="overflow-auto p-8">
        <p class="font-mono text-xs uppercase tracking-[.14em] text-iris">Идея книги</p>
        <h2 class="mt-4 max-w-4xl text-3xl font-semibold leading-[1.3] tracking-[-.025em]">
          {selectedIdea.formulation}
        </h2>
        <div class="mt-7 flex flex-wrap gap-2">
          {#each selectedIdea.assignments as assignment (assignment)}<span
              class="rounded-md border border-iris/20 bg-iris/10 px-3 py-1.5 text-sm"
              >{assignmentLabel(assignment)}</span
            >{/each}<button class="rounded-md border border-dashed border-white/15 px-3 py-1.5 text-sm text-mist-dim"
            >+ Назначение</button
          >
        </div>
        <section class="mt-9">
          <div class="flex items-center justify-between">
            <h3 class="font-semibold">Источники</h3>
            <a
              class="text-sm text-amber no-underline"
              href={resolve("/reader/[bookId]", { bookId: selectedIdea.bookId })}>Открыть в книге</a
            >
          </div>
          {#each selectedIdea.fragments as fragment (`${fragment.page}-${fragment.excerpt}`)}<blockquote
              class="mt-4 border-l-2 border-amber bg-night/30 p-5"
            >
              <p class="leading-7">{fragment.excerpt}</p>
              <footer class="mt-3 font-mono text-xs text-amber">
                {bookForIdea(selectedIdea)?.title} · стр. {fragment.page}
              </footer>
            </blockquote>{/each}
        </section>
        <section class="mt-8 grid grid-cols-2 gap-5">
          <div class="rounded-lg border border-white/8 p-5">
            <h3 class="font-semibold">Практика</h3>
            <p class="mt-3 text-sm leading-6 text-mist-dim">
              {library!.experiments.find((experiment) => experiment.ideaId === selectedIdea.id)?.situation ??
                "Практический эксперимент ещё не создан."}
            </p>
          </div>
          <div class="rounded-lg border border-white/8 p-5">
            <h3 class="font-semibold">Связи</h3>
            <p class="mt-3 text-sm text-mist-dim">Дополняет идею о короткой петле обратной связи.</p>
          </div>
        </section>
        <section class="mt-8 border-t border-white/8 pt-6">
          <div class="flex items-center gap-3">
            <Sparkles class="size-5 text-iris" />
            <h3 class="font-semibold">Проверка идеи Codex</h3>
            <span class="ml-auto font-mono text-[10px] uppercase text-mist-faint">Только по явному запросу</span>
          </div>
          <p class="mt-3 max-w-3xl text-sm leading-6 text-mist-dim">
            Перед запуском вы увидите пакет из инструкции, этого источника и своей формулировки. Другие записи и PDF
            целиком не передаются.
          </p>
          <Button>Подготовить проверку</Button>
        </section>
      </article>{/if}
  </section>
{/snippet}

{#snippet practiceView()}
  <div class="grid grid-cols-[minmax(0,.85fr)_minmax(420px,1.15fr)] gap-6">
    <section class="rounded-xl border border-white/8 bg-slate p-7">
      <p class="font-mono text-xs uppercase tracking-[.14em] text-iris">Восстановление знания</p>
      <h2 class="mt-3 text-2xl font-semibold">Что меняется при отказе лидера?</h2>
      <p class="mt-3 leading-7 text-mist-dim">
        Опишите подходящую ситуацию, объясните идею своими словами и назовите ограничения.
      </p>
      <div class="mt-6"><TextArea id="recall-answer" label="Мой ответ и ограничения" bind:value={recallAnswer} /></div>
      {#if !recallRevealed}<Button
          variant="primary"
          disabled={!recallAnswer.trim()}
          onclick={() => (recallRevealed = true)}>Свериться с идеей</Button
        >{:else}<div class="mt-5 rounded-lg border border-amber/30 bg-amber/[.06] p-5">
          <small class="font-mono uppercase text-amber">Исходная идея и источник</small>
          <p class="mt-3 leading-7">{library!.ideas[0]?.formulation}</p>
        </div>
        <div class="mt-5">
          <p class="mb-3 text-sm font-semibold">Как удалось восстановить?</p>
          <div class="flex gap-2">
            <Button>Уверенно</Button><Button>Частично</Button><Button>Не восстановил</Button>
          </div>
          <p class="mt-4 text-sm text-mist-dim">
            Следующее восстановление предложено на 3 августа. Его можно перенести или запустить раньше.
          </p>
        </div>{/if}
    </section>
    <section class="rounded-xl border border-white/8 bg-slate p-7">
      <div class="flex items-start justify-between">
        <div>
          <p class="font-mono text-xs uppercase tracking-[.14em] text-amber">Практический эксперимент</p>
          <h2 class="mt-3 text-2xl font-semibold">Явная смена владельца журнала</h2>
        </div>
        <span class="rounded-md bg-iris/12 px-3 py-1.5 font-mono text-xs text-iris"
          >{experimentStep === "running" ? "Выполняется" : "Подведение итогов"}</span
        >
      </div>
      <p class="mt-5 text-sm text-mist-dim">Designing Data-Intensive Applications · идея о риске единого лидера</p>
      <div class="mt-7 grid grid-cols-[22px_1fr] gap-x-4 gap-y-6">
        {@render step("Замысел", true, "Проверить, делает ли явная аренда отказ понятнее команде.")}{@render step(
          "Выполняется",
          true,
          "Применяем переход состояния в журнале конфигурации.",
        )}{@render step(
          "Подведение итогов",
          experimentStep === "review",
          "Зафиксировать наблюдаемый результат и авторский вывод.",
        )}{@render step("Завершён", false, "Положительный результат не обязателен.")}
      </div>
      <div class="mt-8 flex gap-2">
        <Button variant="primary" onclick={() => (experimentStep = "review")}>Перейти к итогу</Button><Button
          >Записать следующий шаг</Button
        ><Button>Отменить с причиной</Button>
      </div>
    </section>
  </div>
{/snippet}

{#snippet step(label: string, active: boolean, detail: string)}<span
    class="mt-1 grid size-[22px] place-items-center rounded-full border {active
      ? 'border-iris-strong bg-iris-strong text-white'
      : 'border-white/15 text-transparent'}"><Check class="size-3" /></span
  >
  <div>
    <b class="text-sm">{label}</b>
    <p class="mt-1 text-sm leading-6 text-mist-dim">{detail}</p>
  </div>{/snippet}

{#snippet completionView()}
  <div class="mx-auto max-w-5xl">
    <div class="mb-8 flex items-center justify-between">
      {#each ["Чтение", "Идеи", "Ретроспектива", "Работа", "Эксперименты", "Подтверждение"] as label, index (label)}<div
          class="flex items-center {index < 5 ? 'flex-1' : ''}"
        >
          <span
            class="grid size-8 place-items-center rounded-full border font-mono text-xs {completionStep >= index + 1
              ? 'border-iris-strong bg-iris-strong text-white'
              : 'border-white/12 text-mist-dim'}">{index + 1}</span
          >{#if index < 5}<span class="mx-2 h-px flex-1 bg-white/10"></span>{/if}
        </div>{/each}
    </div>
    <section class="rounded-xl border border-white/8 bg-slate p-8">
      <p class="font-mono text-xs uppercase tracking-[.14em] text-iris">Шаг {completionStep} из 6</p>
      {#if completionStep === 1}<h2 class="mt-3 text-3xl font-semibold">Чтение действительно завершено?</h2>
        <p class="mt-4 max-w-2xl leading-7 text-mist-dim">
          Это фиксирует окончание работы с текстом, но ещё не завершает изучение книги.
        </p>
        <div class="mt-7">
          <Button variant="primary" onclick={() => (completionStep = 2)}>Чтение завершено</Button>
        </div>{:else if completionStep === 2}<h2 class="mt-3 text-3xl font-semibold">Выберите 3–7 значимых идей</h2>
        <div class="mt-6 grid gap-3">
          {#each library!.ideas as idea (idea.id)}<CheckboxField
              id={`significant-${idea.id}`}
              label={idea.formulation}
              checked={significantIdeas.includes(idea.id)}
              onCheckedChange={(checked) => toggleSignificantIdea(idea.id, checked)}
            />{/each}
        </div>
        <div class="mt-7">
          <Button variant="primary" onclick={() => (completionStep = 3)}>Продолжить</Button>
        </div>{:else if completionStep === 3}<h2 class="mt-3 text-3xl font-semibold">Ретроспектива книги</h2>
        <p class="mt-3 text-mist-dim">Что изменилось в вашем понимании или действиях? Итог пишете вы.</p>
        <div class="mt-6"><TextArea id="retrospective" label="Авторский итог" bind:value={retrospective} /></div>
        <Button variant="primary" disabled={!retrospective.trim()} onclick={() => (completionStep = 4)}
          >Сохранить черновик и продолжить</Button
        >{:else}<h2 class="mt-3 text-3xl font-semibold">Незавершённая работа сохранится</h2>
        <p class="mt-4 leading-7 text-mist-dim">
          Черновики, проверки и восстановления требуют вашего решения. Продолжающиеся эксперименты останутся активными
          после завершения изучения.
        </p>
        <div class="mt-6 rounded-lg border border-white/8 bg-night/30 p-5">
          <p>Черновики <span class="float-right font-mono">{library!.drafts.length}</span></p>
          <p class="mt-3">
            Продолжающиеся эксперименты <span class="float-right font-mono"
              >{library!.experiments.filter((item) => !item.completed).length}</span
            >
          </p>
        </div>
        <div class="mt-7">
          <Button variant="primary" onclick={() => (completionStep = Math.min(6, completionStep + 1))}
            >{completionStep === 6 ? "Завершить изучение" : "Продолжить"}</Button
          >
        </div>{/if}
    </section>
  </div>
{/snippet}

{#snippet settingsView()}
  <div class="grid grid-cols-[220px_minmax(0,1fr)] gap-7">
    <nav aria-label="Разделы настроек" class="grid content-start gap-1 rounded-xl border border-white/8 bg-slate p-3">
      {#each settingsNavigation as item (item.label)}{@const Icon = item.icon}<button
          class="flex items-center gap-3 rounded-md px-3 py-3 text-left text-sm text-mist-dim first:bg-iris/12 first:text-mist"
          ><Icon class="size-4" />{item.label}</button
        >{/each}
    </nav>
    <div class="grid gap-5">
      <section class="rounded-xl border border-white/8 bg-slate p-7">
        <p class="font-mono text-xs uppercase tracking-[.14em] text-iris">Интерфейс</p>
        <h2 class="mt-3 text-xl font-semibold">Чтение и рабочее пространство</h2>
        <div class="mt-6 grid gap-5">
          <SelectField
            label="Режим документа по умолчанию"
            value={readerMode}
            options={[
              { value: "muted", label: "Приглушённый светлый" },
              { value: "original", label: "Оригинальный" },
              { value: "dark", label: "Тёмный инвертированный" },
            ]}
            onValueChange={(value) => (readerMode = value)}
          /><CheckboxField
            id="invert-images"
            label="Инвертировать изображения в тёмном режиме"
            bind:checked={readerImages}
          />
        </div>
      </section>
      <section class="rounded-xl border border-white/8 bg-slate p-7">
        <div class="flex items-start justify-between">
          <div>
            <p class="font-mono text-xs uppercase tracking-[.14em] text-amber">Резервные копии</p>
            <h2 class="mt-3 text-xl font-semibold">Локальное восстановление</h2>
          </div>
          <span class="font-mono text-xs text-success">Snapshot сегодня, 18:40</span>
        </div>
        <div class="mt-6 grid grid-cols-2 gap-4">
          <div class="rounded-lg border border-white/8 bg-night/30 p-5">
            <b>Автоматический snapshot</b>
            <p class="mt-2 text-sm leading-6 text-mist-dim">Последняя внутренняя копия: 30 июля 2026, 18:40.</p>
            <Button>Восстановить последний</Button>
          </div>
          <div class="rounded-lg border border-white/8 bg-night/30 p-5">
            <b>Переносимый архив</b>
            <p class="mt-2 text-sm leading-6 text-mist-dim">Последний экспорт: 12 июля 2026. Архив защищён паролем.</p>
            <div class="flex gap-2"><Button>Экспортировать</Button><Button>Импортировать</Button></div>
          </div>
        </div>
      </section>
      <section class="rounded-xl border border-white/8 bg-slate p-7">
        <div class="flex items-center">
          <div>
            <b>Обновления Bookshelf</b>
            <p class="mt-1 text-sm text-mist-dim">Версия 0.1.0 · проверка выполняется только явно.</p>
          </div>
          <Button>Проверить обновления</Button>
        </div>
      </section>
      <section class="rounded-xl border border-white/8 bg-slate p-7">
        <b>Диагностический журнал</b>
        <p class="mt-2 text-sm text-mist-dim">
          Хранится ограниченное время локально. Автоматической отправки и телеметрии нет.
        </p>
        <Button>Экспортировать журнал</Button>
      </section>
    </div>
  </div>
{/snippet}

{#snippet readerView()}
  <div class="flex h-screen flex-col overflow-hidden bg-night text-mist" data-testid="reader-ready">
    <header class="z-20 flex min-h-16 items-center gap-3 border-b border-white/8 bg-graphite px-4">
      <a
        href={resolve("/library/[bookId]", { bookId: selectedBook?.id ?? "book" })}
        aria-label="Вернуться к книге"
        class="grid size-10 place-items-center rounded-md text-mist-dim no-underline hover:bg-slate hover:text-mist"
        ><ArrowLeft class="size-5" /></a
      >
      <div class="min-w-0">
        <b class="block truncate text-sm">{selectedBook?.title ?? "Режим чтения"}</b><small
          class="block truncate text-mist-dim">Глава 5 · Репликация</small
        >
      </div>
      <span class="ml-4 font-mono text-xs text-mist-dim">стр. {readerPage} / 612</span>
      <div class="ml-auto flex items-center gap-1">
        <button
          aria-label="Уменьшить масштаб"
          class="grid size-10 place-items-center rounded-md hover:bg-slate"
          onclick={() => (readerZoom = Math.max(0.5, readerZoom - 0.1))}><ZoomOut class="size-4" /></button
        ><span class="w-14 text-center font-mono text-xs">{Math.round(readerZoom * 100)}%</span><button
          aria-label="Увеличить масштаб"
          class="grid size-10 place-items-center rounded-md hover:bg-slate"
          onclick={() => (readerZoom = Math.min(2, readerZoom + 0.1))}><ZoomIn class="size-4" /></button
        ><button
          aria-label="Поиск в книге"
          class="ml-2 grid size-10 place-items-center rounded-md hover:bg-slate"
          onclick={() => setReaderSidebar("search")}><Search class="size-4" /></button
        ><span role="status" class="ml-2 w-24 font-mono text-[11px] text-mist-dim"
          >{saveState === "saving" ? "Сохранение…" : saveState === "error" ? "Не сохранено" : "Сохранено"}</span
        ><button
          {@attach rememberSidebarTrigger}
          aria-label="Показать инструменты чтения"
          aria-expanded={readerSidebar !== null}
          class="grid size-10 place-items-center rounded-md data-[open=true]:bg-iris/15 data-[open=true]:text-iris"
          data-open={readerSidebar !== null}
          onclick={() => (readerSidebar ? closeReaderSidebar() : setReaderSidebar("note"))}
          ><PanelRight class="size-5" /></button
        >
      </div>
    </header>
    <div
      class="grid min-h-0 flex-1 transition-[grid-template-columns] duration-200 max-[1280px]:block"
      style:grid-template-columns={readerSidebar ? `minmax(0,1fr) ${readerSidebarWidth}px` : "minmax(0,1fr) 0px"}
    >
      {#if readerDocumentUrl}<main class="reader-document min-w-0 overflow-hidden">
          <ContinuousPdfReader
            url={readerDocumentUrl}
            initialPage={readerPage}
            initialScroll={selectedBook?.reading.scroll ?? 0}
            zoom={readerZoom}
            mode={readerMode as "muted" | "original" | "dark"}
            invertImages={readerImages}
            onPosition={savePdfPosition}
            onSelection={capturePdfSelection}
            onOutline={(outline) => {
              if (selectedBook && outline.length && selectedBook.outline.length === 0)
                void run({ kind: "saveOutline", bookId: selectedBook.id, outline }, "");
            }}
          />
        </main>{:else}<main class="reader-document min-w-0 overflow-y-auto bg-[#15191f] px-12 py-8 max-[1280px]:px-6">
          <div class="mx-auto grid max-w-[950px] gap-5">
            {#each [readerPage - 1, readerPage, readerPage + 1, readerPage + 2] as page (page)}<article
                class="relative mx-auto min-h-[1040px] w-full max-w-[780px] border border-black/20 px-[82px] py-[72px] text-[#252a31] shadow-[0_16px_45px_rgba(0,0,0,.28)] {readerMode ===
                'dark'
                  ? 'bg-[#d7d9db] invert'
                  : readerMode === 'original'
                    ? 'bg-white'
                    : 'bg-[#e9e7e0]'}"
                data-page={page}
              >
                <span class="absolute left-5 top-8 font-mono text-[10px] text-[#62676c]">{page}</span>
                <div
                  class="absolute -right-8 inset-y-0 w-5 border-l border-amber/30"
                  aria-label="Цифровое поле источников"
                >
                  {#if page === readerPage}<button
                      aria-label="Источник черновой заметки на странице {page}"
                      class="absolute top-[38%] grid size-4 -translate-x-1/2 place-items-center rounded-full border border-amber bg-[#40331f] text-[8px] text-amber"
                      >1</button
                    >{/if}
                </div>
                <p class="mb-4 font-mono text-[10px] uppercase tracking-[.16em] text-[#666]">Chapter 5 · Replication</p>
                <h2 class="mb-8 text-[28px] font-semibold tracking-tight">Leader-based replication</h2>
                <p class="mb-5 text-[17px] leading-[1.82]">
                  Storing a copy of the same data on several different nodes can keep systems available when individual
                  machines fail. The difficult part is not copying bytes, but deciding which order of writes every
                  replica must observe.
                </p>
                <p
                  class="mb-5 text-[17px] leading-[1.82] {page === readerPage
                    ? 'rounded-sm bg-amber/25 outline outline-1 outline-amber/50'
                    : ''}"
                >
                  The advantage of a leader-based approach is that conflict resolution happens on the leader. Followers
                  apply the same stream of changes, while failover decides which history may continue.
                </p>
                <p class="mb-5 text-[17px] leading-[1.82]">
                  This simplification has a cost. Leadership is not merely an infrastructure role: changing it changes
                  which writes are accepted, how stale reads are interpreted, and where recovery begins.
                </p>
                <div class="my-10 grid grid-cols-3 gap-4 rounded border border-[#777]/30 p-5">
                  <div class="rounded bg-[#657185] p-4 text-center text-white">Leader</div>
                  <div class="rounded bg-[#aeb4bd] p-4 text-center">Follower A</div>
                  <div class="rounded bg-[#aeb4bd] p-4 text-center">Follower B</div>
                </div>
                <p class="text-[17px] leading-[1.82]">
                  A robust design therefore makes the transition visible in the model and tests the boundaries around
                  it, rather than treating failover as an invisible operational concern.
                </p>
              </article>{/each}
          </div>
        </main>{/if}
      <aside
        class="z-30 min-w-0 overflow-hidden border-l border-white/8 bg-graphite max-[1280px]:fixed max-[1280px]:inset-y-16 max-[1280px]:right-0 max-[1280px]:w-[min(400px,calc(100vw-2rem))] max-[1280px]:shadow-2xl {readerSidebar
          ? ''
          : 'max-[1280px]:translate-x-full'}"
        aria-label="Инструменты чтения"
      >
        {#if readerSidebar}<div class="flex h-full flex-col" style:width={`${readerSidebarWidth}px`}>
            <div class="flex items-center border-b border-white/8 p-2">
              {#each readerTabs as tab (tab.id)}{@const TabIcon = tab.icon}<button
                  class="flex min-h-10 flex-1 items-center justify-center gap-2 rounded-md text-xs text-mist-dim data-[active=true]:bg-iris/12 data-[active=true]:text-mist"
                  data-active={readerSidebar === tab.id}
                  onclick={() => setReaderSidebar(tab.id)}><TabIcon class="size-4" />{tab.label}</button
                >{/each}<button
                aria-label="Закрыть инструменты"
                class="ml-1 grid size-10 place-items-center rounded-md hover:bg-slate"
                onclick={closeReaderSidebar}><X class="size-4" /></button
              >
            </div>
            <div class="min-h-0 flex-1 overflow-auto p-5">
              {#if readerSidebar === "note"}<p class="font-mono text-[10px] uppercase tracking-[.14em] text-amber">
                  Источник · стр. {readerPage}
                </p>
                <h2 class="mt-2 text-xl font-semibold">Черновая заметка</h2>
                <p class="mt-2 text-sm leading-6 text-mist-dim">
                  Выделите текст в документе. Источник сохранится отдельно от вашей мысли.
                </p>
                <div class="mt-5 grid gap-4">
                  <TextArea id="reader-excerpt" label="Фрагмент книги" bind:value={readerExcerpt} required /><TextArea
                    id="reader-comment"
                    label="Моя мысль (необязательно)"
                    bind:value={readerComment}
                  /><Button variant="primary" onclick={saveReaderDraft} disabled={!readerExcerpt.trim()}
                    >В черновики · Ctrl+Enter</Button
                  ><Button>Оформить как идею</Button>
                </div>
                <div class="mt-7 border-t border-white/8 pt-5">
                  <h3 class="text-sm font-semibold">Последние заметки книги</h3>
                  {#each library?.drafts
                    .filter((draft) => draft.bookId === selectedBook?.id)
                    .slice(0, 5) ?? [] as draft (draft.id)}<p
                      class="mt-3 line-clamp-2 border-l border-amber pl-3 text-xs leading-5 text-mist-dim"
                    >
                      {draft.excerpt}
                    </p>{/each}
                </div>{:else if readerSidebar === "outline"}<h2 class="text-xl font-semibold">Оглавление</h2>
                <div class="mt-5 grid gap-1">
                  {#each selectedBook?.outline ?? [] as item (`${item.page}-${item.title}`)}<button
                      class="flex rounded-md px-3 py-3 text-left text-sm hover:bg-slate"
                      onclick={() => saveReaderPosition(item.page)}
                      ><span>{item.title}</span><span class="ml-auto font-mono text-mist-dim">{item.page}</span></button
                    >{/each}
                </div>{:else}<h2 class="text-xl font-semibold">Поиск в книге</h2>
                <div class="mt-5">
                  <TextField
                    id="reader-search"
                    label="Текстовый слой"
                    bind:value={readerSearch}
                    placeholder="Найти без учёта регистра"
                  />
                </div>
                {#if readerSearch}<p class="mt-5 font-mono text-xs text-mist-dim">3 результата · 1 из 3</p>
                  <button class="mt-3 rounded-lg border border-white/8 bg-slate p-4 text-left text-sm"
                    ><mark class="bg-amber/30 text-mist">{readerSearch}</mark> в потоке изменений лидера…</button
                  >{/if}{/if}
            </div>
            <label class="border-t border-white/8 p-3 text-[10px] text-mist-faint"
              >Ширина панели <input
                aria-label="Ширина панели"
                type="range"
                min="320"
                max="560"
                bind:value={readerSidebarWidth}
                onchange={persistReaderPreferences}
              /></label
            >
          </div>{/if}
      </aside>
    </div>
  </div>
{/snippet}
