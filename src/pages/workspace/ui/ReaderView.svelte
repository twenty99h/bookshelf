<script lang="ts">
  import { resolve } from "$app/paths";
  import { ArrowLeft, ListTree, PanelRight, Search, StickyNote, X, ZoomIn, ZoomOut } from "@lucide/svelte";
  import { Button, TextArea, TextField } from "@/shared/ui";
  import type { Book, LibraryState, OutlineItem, SourceFragment } from "@/shared/api";
  import ContinuousPdfReader from "./reader/ContinuousPdfReader.svelte";

  type ReaderSidebar = "note" | "outline" | "search" | null;
  type DocumentMode = "muted" | "original" | "dark";
  type SearchResult = { page: number; excerpt: string };

  let {
    library,
    selectedBook,
    documentUrl,
    page = $bindable(),
    zoom = $bindable(),
    mode = $bindable(),
    images = $bindable(),
    sidebar = $bindable(),
    sidebarWidth = $bindable(),
    search = $bindable(),
    searchResults = $bindable(),
    saveState,
    onChangeZoom,
    onSetSidebar,
    onCloseSidebar,
    rememberSidebarTrigger,
    onSavePosition,
    onPdfPosition,
    onSourceSelect,
    onSearchResults,
    onSaveOutline,
    onSaveDraft,
    onCreateIdea,
    onPersistPreferences,
  }: {
    library: LibraryState | null;
    selectedBook: Book | null;
    documentUrl: string | null;
    page: number;
    zoom: number;
    mode: DocumentMode;
    images: boolean;
    sidebar: ReaderSidebar;
    sidebarWidth: number;
    search: string;
    searchResults: SearchResult[];
    saveState: "saved" | "saving" | "error";
    onChangeZoom: (delta: number) => void;
    onSetSidebar: (tab: Exclude<ReaderSidebar, null>) => void;
    onCloseSidebar: () => void;
    rememberSidebarTrigger: (node: HTMLButtonElement) => () => void;
    onSavePosition: (page?: number) => Promise<void>;
    onPdfPosition: (page: number, scroll: number) => void;
    onSourceSelect: (draftId: string, source: SourceFragment) => void;
    onSearchResults: (results: SearchResult[]) => void;
    onSaveOutline: (outline: OutlineItem[]) => void;
    onSaveDraft: (excerpt: string, comment: string, fragments: SourceFragment[]) => Promise<string | null>;
    onCreateIdea: (draftId: string, formulation: string) => Promise<boolean>;
    onPersistPreferences: () => Promise<void>;
  } = $props();

  let excerpt = $state("");
  let comment = $state("");
  let fragments = $state<SourceFragment[]>([]);
  let ideaDraftId = $state("");
  let ideaFormulation = $state("");

  const tabs: { id: Exclude<ReaderSidebar, null>; label: string; icon: typeof StickyNote }[] = [
    { id: "note", label: "Заметка", icon: StickyNote },
    { id: "outline", label: "Оглавление", icon: ListTree },
    { id: "search", label: "Поиск", icon: Search },
  ];
  const captureAllowed = $derived(selectedBook?.studyStatus !== "completed");
  const currentChapter = $derived.by(
    () =>
      (selectedBook?.outline ?? [])
        .filter((item) => item.page <= page)
        .toSorted((a, b) => b.page - a.page)
        .at(0)?.title ?? "Без раздела",
  );

  async function selectDocumentMode(nextMode: DocumentMode) {
    mode = nextMode;
    await onPersistPreferences();
  }

  async function toggleImageInversion() {
    images = !images;
    await onPersistPreferences();
  }

  function captureSelection(selection: SourceFragment[]) {
    const first = selection[0];
    if (!first) return;
    page = first.page;
    fragments = selection;
    excerpt = selection.map((fragment) => fragment.excerpt).join("\n");
    onSetSidebar("note");
  }

  async function saveDraft() {
    const draftId = await onSaveDraft(excerpt, comment, fragments);
    if (draftId) {
      excerpt = "";
      comment = "";
      fragments = [];
    }
    return draftId;
  }

  async function startIdea() {
    const draftId = await saveDraft();
    if (draftId) ideaDraftId = draftId;
  }

  async function createIdea() {
    if (await onCreateIdea(ideaDraftId, ideaFormulation)) {
      ideaDraftId = "";
      ideaFormulation = "";
    }
  }
</script>

<svelte:window
  onkeydown={(event) => {
    if (event.ctrlKey && event.key === "Enter" && sidebar === "note") {
      event.preventDefault();
      void saveDraft();
    }
  }}
/>

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
        class="block truncate text-mist-dim">{currentChapter}</small
      >
    </div>
    <span class="ml-4 font-mono text-xs text-mist-dim">стр. {page} / {selectedBook?.pageCount ?? "—"}</span>
    <div class="ml-auto flex items-center gap-1">
      <div
        class="mr-2 flex items-center rounded-md border border-white/10 p-0.5"
        role="group"
        aria-label="Режим документа"
      >
        <button
          aria-label="Приглушённый светлый режим"
          aria-pressed={mode === "muted"}
          class="min-h-8 rounded px-2 text-xs text-mist-dim aria-pressed:bg-white/10 aria-pressed:text-mist"
          onclick={() => selectDocumentMode("muted")}>Светлый</button
        ><button
          aria-label="Оригинальный режим"
          aria-pressed={mode === "original"}
          class="min-h-8 rounded px-2 text-xs text-mist-dim aria-pressed:bg-white/10 aria-pressed:text-mist"
          onclick={() => selectDocumentMode("original")}>Оригинал</button
        ><button
          aria-label="Тёмный инвертированный режим"
          aria-pressed={mode === "dark"}
          class="min-h-8 rounded px-2 text-xs text-mist-dim aria-pressed:bg-white/10 aria-pressed:text-mist"
          onclick={() => selectDocumentMode("dark")}>Тёмный</button
        >
      </div>
      <button
        aria-label={images ? "Не инвертировать изображения" : "Инвертировать изображения"}
        aria-pressed={images}
        class="min-h-9 rounded-md border border-white/10 px-2 text-xs text-mist-dim aria-pressed:text-amber"
        onclick={toggleImageInversion}>Схемы</button
      >
      <button
        aria-label="Уменьшить масштаб"
        class="grid size-10 place-items-center rounded-md hover:bg-slate"
        onclick={() => onChangeZoom(-0.1)}><ZoomOut class="size-4" /></button
      ><span class="w-14 text-center font-mono text-xs">{Math.round(zoom * 100)}%</span><button
        aria-label="Увеличить масштаб"
        class="grid size-10 place-items-center rounded-md hover:bg-slate"
        onclick={() => onChangeZoom(0.1)}><ZoomIn class="size-4" /></button
      ><button
        aria-label="Поиск в книге"
        class="ml-2 grid size-10 place-items-center rounded-md hover:bg-slate"
        onclick={() => onSetSidebar("search")}><Search class="size-4" /></button
      >{#if saveState === "error"}<span role="status" class="ml-2 font-mono text-[11px] text-danger">Не сохранено</span
        ><button
          class="rounded px-2 py-1 font-mono text-[11px] text-danger hover:bg-danger/10"
          onclick={() => onSavePosition()}>Повторить сохранение</button
        >{:else}<span role="status" class="ml-2 w-24 font-mono text-[11px] text-mist-dim"
          >{saveState === "saving" ? "Сохранение…" : "Сохранено"}</span
        >{/if}<button
        {@attach rememberSidebarTrigger}
        aria-label="Показать инструменты чтения"
        aria-expanded={sidebar !== null}
        class="grid size-10 place-items-center rounded-md data-[open=true]:bg-iris/15 data-[open=true]:text-iris"
        data-open={sidebar !== null}
        onclick={() => (sidebar ? onCloseSidebar() : onSetSidebar("note"))}><PanelRight class="size-5" /></button
      >
    </div>
  </header>
  <div
    class="grid min-h-0 flex-1 transition-[grid-template-columns] duration-200 max-[1280px]:block"
    style:grid-template-columns={sidebar ? `minmax(0,1fr) ${sidebarWidth}px` : "minmax(0,1fr) 0px"}
  >
    {#if documentUrl}<main class="reader-document min-w-0 overflow-hidden">
        <ContinuousPdfReader
          url={documentUrl}
          initialPage={page}
          initialScroll={selectedBook?.reading.scroll ?? 0}
          {zoom}
          {mode}
          invertImages={images}
          onPosition={onPdfPosition}
          onSelection={captureSelection}
          searchQuery={search}
          onSearchResults={(results) => {
            searchResults = results;
            onSearchResults(results);
          }}
          sources={library?.drafts
            .filter((draft) => draft.bookId === selectedBook?.id)
            .flatMap((draft) => draft.fragments.map((fragment) => ({ draftId: draft.id, fragment }))) ?? []}
          {onSourceSelect}
          onOutline={onSaveOutline}
        />
      </main>{:else}<main class="reader-document min-w-0 overflow-y-auto bg-[#15191f] px-12 py-8 max-[1280px]:px-6">
        <div class="mx-auto grid max-w-[950px] gap-5">
          {#each [page - 1, page, page + 1, page + 2] as documentPage (documentPage)}<article
              class="relative mx-auto min-h-[1040px] w-full max-w-[780px] border border-black/20 px-[82px] py-[72px] text-[#252a31] shadow-[0_16px_45px_rgba(0,0,0,.28)] {mode ===
              'dark'
                ? 'bg-[#d7d9db] invert'
                : mode === 'original'
                  ? 'bg-white'
                  : 'bg-[#e9e7e0]'}"
              data-page={documentPage}
            >
              <span class="absolute left-5 top-8 font-mono text-[10px] text-[#62676c]">{documentPage}</span>
              <div
                class="absolute -right-8 inset-y-0 w-5 border-l border-amber/30"
                aria-label="Цифровое поле источников"
              >
                {#if documentPage === page}<button
                    aria-label="Источник черновой заметки на странице {documentPage}"
                    class="absolute top-[38%] grid size-4 -translate-x-1/2 place-items-center rounded-full border border-amber bg-[#40331f] text-[8px] text-amber"
                    >1</button
                  >{/if}
              </div>
              <p class="mb-4 font-mono text-[10px] uppercase tracking-[.16em] text-[#666]">Chapter 5 · Replication</p>
              <h2 class="mb-8 text-[28px] font-semibold tracking-tight">Leader-based replication</h2>
              <p class="mb-5 text-[17px] leading-[1.82]">
                Storing a copy of the same data on several different nodes can keep systems available when individual
                machines fail. The difficult part is not copying bytes, but deciding which order of writes every replica
                must observe.
              </p>
              <p
                class="mb-5 text-[17px] leading-[1.82] {documentPage === page
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
                A robust design therefore makes the transition visible in the model and tests the boundaries around it,
                rather than treating failover as an invisible operational concern.
              </p>
            </article>{/each}
        </div>
      </main>{/if}
    <aside
      class="z-30 min-w-0 overflow-hidden border-l border-white/8 bg-graphite max-[1280px]:fixed max-[1280px]:inset-y-16 max-[1280px]:right-0 max-[1280px]:w-[min(400px,calc(100vw-2rem))] max-[1280px]:shadow-2xl {sidebar
        ? ''
        : 'max-[1280px]:translate-x-full'}"
      aria-label="Инструменты чтения"
    >
      {#if sidebar}<div class="flex h-full flex-col" style:width={`${sidebarWidth}px`}>
          <div class="flex items-center border-b border-white/8 p-2">
            {#each tabs as tab (tab.id)}{@const TabIcon = tab.icon}<button
                class="flex min-h-10 flex-1 items-center justify-center gap-2 rounded-md text-xs text-mist-dim data-[active=true]:bg-iris/12 data-[active=true]:text-mist"
                data-active={sidebar === tab.id}
                onclick={() => onSetSidebar(tab.id)}><TabIcon class="size-4" />{tab.label}</button
              >{/each}<button
              aria-label="Закрыть инструменты"
              class="ml-1 grid size-10 place-items-center rounded-md hover:bg-slate"
              onclick={onCloseSidebar}><X class="size-4" /></button
            >
          </div>
          <div class="min-h-0 flex-1 overflow-auto p-5">
            {#if sidebar === "note"}<p class="font-mono text-[10px] uppercase tracking-[.14em] text-amber">
                Источник · стр. {page}
              </p>
              <h2 class="mt-2 text-xl font-semibold">Черновая заметка</h2>
              <p class="mt-2 text-sm leading-6 text-mist-dim">
                {!captureAllowed
                  ? "Книга доступна для справочного чтения. Начните повторное изучение на странице книги, чтобы создавать новые черновые заметки."
                  : selectedBook?.hasTextLayer
                    ? "Выделите текст в документе. Источник сохранится отдельно от вашей мысли."
                    : "В этом PDF нет текстового слоя. Укажите фрагмент вручную — заметка сохранится с явной страницей. OCR не выполняется."}
              </p>
              <div class="mt-5 grid gap-4">
                <TextArea id="reader-excerpt" label="Фрагмент книги" bind:value={excerpt} required /><TextArea
                  id="reader-comment"
                  label="Моя мысль (необязательно)"
                  bind:value={comment}
                /><Button variant="primary" onclick={saveDraft} disabled={!captureAllowed || !excerpt.trim()}
                  >В черновики · Ctrl+Enter</Button
                ><Button onclick={startIdea} disabled={!captureAllowed || !excerpt.trim()}>Оформить как идею</Button>
                {#if ideaDraftId}<div class="rounded-lg border border-iris/30 bg-iris/8 p-4">
                    <TextArea
                      id="reader-idea-formulation"
                      label="Моя формулировка идеи"
                      bind:value={ideaFormulation}
                      required
                    />
                    <Button variant="primary" onclick={createIdea} disabled={!ideaFormulation.trim()}
                      >Создать идею</Button
                    >
                    <p class="mt-3 text-xs leading-5 text-mist-dim">
                      Источник уже сохранён. Восстановление, практика или передача назначаются отдельно.
                    </p>
                  </div>{/if}
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
              </div>{:else if sidebar === "outline"}<h2 class="text-xl font-semibold">Оглавление</h2>
              <div class="mt-5 grid gap-1">
                {#each selectedBook?.outline ?? [] as item (`${item.page}-${item.title}`)}<button
                    class="flex rounded-md px-3 py-3 text-left text-sm hover:bg-slate"
                    onclick={() => onSavePosition(item.page)}
                    ><span>{item.title}</span><span class="ml-auto font-mono text-mist-dim">{item.page}</span></button
                  >{/each}
              </div>{:else}<h2 class="text-xl font-semibold">Поиск в книге</h2>
              <div class="mt-5">
                <TextField
                  id="reader-search"
                  label="Текстовый слой"
                  bind:value={search}
                  placeholder="Найти без учёта регистра"
                />
              </div>
              {#if search}<p class="mt-5 font-mono text-xs text-mist-dim">
                  {searchResults.length} результатов
                </p>
                {#each searchResults as result (`${result.page}-${result.excerpt}`)}<button
                    class="mt-3 rounded-lg border border-white/8 bg-slate p-4 text-left text-sm"
                    onclick={() => onSavePosition(result.page)}
                    ><span class="font-mono text-amber">стр. {result.page}</span> {result.excerpt}</button
                  >{/each}{/if}{/if}
          </div>
          <label class="border-t border-white/8 p-3 text-[10px] text-mist-faint"
            >Ширина панели <input
              aria-label="Ширина панели"
              type="range"
              min="320"
              max="560"
              bind:value={sidebarWidth}
              onchange={onPersistPreferences}
            /></label
          >
        </div>{/if}
    </aside>
  </div>
</div>
