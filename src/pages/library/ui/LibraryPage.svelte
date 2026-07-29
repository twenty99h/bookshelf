<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import {
    Button,
    NavigationItem,
    PageHeader,
    StatePanel,
    StatusMessage,
    TextField,
    WorkspaceShell,
  } from "@/shared/ui";
  import IdeaWorkbench from "./IdeaWorkbench.svelte";
  import { PdfReader } from "@/features/pdf-reader";
  import {
    bookFilePath,
    commandErrorMessage,
    executeLibraryAction,
    exportDraftMarkdown,
    exportLibraryArchive,
    exportMaterialMarkdown,
    importLibraryArchive,
    installSignedUpdate,
    importPdf,
    loadLibrary,
    saveWorkspaceNote,
    restoreLatestSnapshot,
    searchLibrary,
    startCodexLogin,
    type Book,
    type CodexStreamEvent,
    type LibraryAction,
    type LibraryState,
    type SearchResult,
  } from "@/shared/api";

  type View = "library" | "queue" | "ideas" | "study";
  let library = $state<LibraryState | null>(null);
  let view = $state<View>("library");
  let note = $state("");
  let loading = $state(true);
  let busy = $state(false);
  let saved = $state(false);
  let error = $state("");
  let settingsOpen = $state(false);
  let searchQuery = $state("");
  let results = $state<SearchResult[]>([]);
  let openedSearchResult = $state<SearchResult | null>(null);
  let readingBook = $state<Book | null>(null);
  let bookUrl = $state("");
  let excerpt = $state("");
  let context = $state("");
  let comment = $state("");
  let section = $state("Введение");
  let formulation = $state("");
  let sessionIntention = $state("");
  let topicName = $state("");
  let feedback = $state("");
  let archivePassword = $state("");
  let missedSessionStatus = $state("moved");
  let missedSessionReason = $state("");
  let attachIdeaId = $state("");
  let retrospective = $state("");
  let significantIdeaIds = $state<string[]>([]);
  let continuingWork = $state("");
  let debtDecision = $state("");
  let outlineTitle = $state("");
  let outlinePage = $state(1);
  let positionTimer: ReturnType<typeof setTimeout> | undefined;
  let codexLoginUrl = $state("");
  let codexLoginCode = $state("");
  let codexLoginRunning = $state(false);

  let debt = $derived((library?.drafts.length ?? 0) + (library?.reviews.filter((item) => item.pending).length ?? 0));
  let activeBook = $derived(library?.books.find((book) => book.id === library?.activeStudyBookId));
  let openedIdea = $derived(
    openedSearchResult?.kind === "idea" ? library?.ideas.find((idea) => idea.id === openedSearchResult?.id) : undefined,
  );
  let openedTopic = $derived(
    openedSearchResult?.kind === "topic"
      ? library?.topics.find((topic) => topic.id === openedSearchResult?.id)
      : undefined,
  );
  let openedMaterial = $derived(
    openedSearchResult?.kind === "material"
      ? library?.materials.find((material) => material.id === openedSearchResult?.id)
      : undefined,
  );
  let nextStep = $derived(
    activeBook
      ? `Продолжить «${activeBook.title}»`
      : debt > 0
        ? "Разобрать ближайшую заметку"
        : library?.books.length
          ? "Выбрать книгу для изучения"
          : "Импортировать первую книгу",
  );

  onMount(async () => {
    void listen<CodexStreamEvent>("codex-login-event", (event) => {
      if (event.payload.kind !== "deviceCode") return;
      [codexLoginUrl, codexLoginCode] = event.payload.text.split("\n", 2);
    });
    try {
      library = await loadLibrary();
      note = library.workspaceNote;
    } catch (cause) {
      error = commandErrorMessage(cause);
    } finally {
      loading = false;
    }
  });

  async function loginCodex() {
    codexLoginRunning = true;
    error = "";
    codexLoginUrl = "";
    codexLoginCode = "";
    try {
      await startCodexLogin();
      feedback = "Вход в Codex завершён";
    } catch (cause) {
      error = commandErrorMessage(cause);
    } finally {
      codexLoginRunning = false;
    }
  }

  async function run(action: LibraryAction, success = "Изменения сохранены") {
    busy = true;
    error = "";
    feedback = "";
    try {
      library = await executeLibraryAction(action);
      feedback = success;
    } catch (cause) {
      error = commandErrorMessage(cause);
    } finally {
      busy = false;
    }
  }

  async function importBook() {
    error = "";
    const path = await open({ multiple: false, filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (!path) return;
    busy = true;
    try {
      library = await importPdf(path);
      feedback = "Книга скопирована в личную библиотеку";
      view = "library";
    } catch (cause) {
      error = commandErrorMessage(cause);
    } finally {
      busy = false;
    }
  }

  async function openBook(book: Book) {
    busy = true;
    error = "";
    try {
      bookUrl = convertFileSrc(await bookFilePath(book.id));
      readingBook = book;
    } catch (cause) {
      error = commandErrorMessage(cause);
    } finally {
      busy = false;
    }
  }

  async function savePosition() {
    if (!readingBook) return;
    await run(
      {
        kind: "updateReading",
        bookId: readingBook.id,
        page: readingBook.reading?.page || 1,
        zoom: readingBook.reading?.zoom || 1,
        scroll: readingBook.reading?.scroll || 0,
      },
      "Место чтения сохранено",
    );
  }

  function recordPosition(page: number, zoom: number, scroll: number) {
    if (!readingBook) return;
    readingBook.reading = { page, zoom, scroll };
    if (positionTimer) clearTimeout(positionTimer);
    const bookId = readingBook.id;
    positionTimer = setTimeout(async () => {
      try {
        library = await executeLibraryAction({ kind: "updateReading", bookId, page, zoom, scroll });
        readingBook = library.books.find((book) => book.id === bookId) ?? null;
      } catch (cause) {
        error = commandErrorMessage(cause);
      }
    }, 450);
  }

  async function acceptImportedOutline(importedOutline: Book["outline"]) {
    if (!readingBook || readingBook.outline.length > 0) return;
    readingBook.outline = importedOutline;
    await saveOutline();
  }

  async function saveOutline() {
    if (!readingBook) return;
    await run(
      { kind: "saveOutline", bookId: readingBook.id, outline: readingBook.outline ?? [] },
      "Исправленное оглавление сохранено отдельно от PDF",
    );
  }

  async function addOutlineItem(event: SubmitEvent) {
    event.preventDefault();
    if (!readingBook || !outlineTitle.trim()) return;
    readingBook.outline = [
      ...(readingBook.outline ?? []),
      { id: `outline-${Date.now()}`, title: outlineTitle, page: outlinePage, parentId: null },
    ];
    outlineTitle = "";
    await saveOutline();
  }

  async function captureDraft(event: SubmitEvent) {
    event.preventDefault();
    if (!readingBook) return;
    await run(
      {
        kind: "captureDraft",
        bookId: readingBook.id,
        section,
        page: readingBook.reading?.page || 1,
        excerpt,
        context,
        comment,
      },
      "Черновая заметка добавлена в очередь",
    );
    if (!error) {
      excerpt = "";
      context = "";
      comment = "";
    }
  }

  async function resolveDraft(draftId: string) {
    await run(
      { kind: "resolveDraftAsIdea", draftId, formulation, section, assignments: ["recall"] },
      "Черновик стал самостоятельной идеей",
    );
    if (!error) formulation = "";
  }

  async function saveNote(event: SubmitEvent) {
    event.preventDefault();
    busy = true;
    saved = false;
    error = "";
    try {
      library = await saveWorkspaceNote(note);
      saved = true;
    } catch (cause) {
      error = commandErrorMessage(cause);
    } finally {
      busy = false;
    }
  }

  async function search() {
    openedSearchResult = null;
    try {
      results = await searchLibrary(searchQuery);
    } catch (cause) {
      error = commandErrorMessage(cause);
    }
  }

  async function openSearchResult(result: SearchResult) {
    if (result.kind === "book") {
      const book = library?.books.find((candidate) => candidate.id === result.id);
      if (book) await openBook(book);
      return;
    }
    openedSearchResult = result;
    navigate("ideas");
  }

  function searchResultKind(kind: SearchResult["kind"]) {
    return (
      { book: "Книга", idea: "Идея", topic: "Тема", material: "Материал" } satisfies Record<
        SearchResult["kind"],
        string
      >
    )[kind];
  }

  async function exportArchive() {
    const path = await save({
      defaultPath: "bookshelf-library.age",
      filters: [{ name: "Bookshelf archive", extensions: ["age"] }],
    });
    if (!path) return;
    busy = true;
    error = "";
    try {
      await exportLibraryArchive(path, archivePassword);
      feedback = "Зашифрованный архив сохранён";
    } catch (cause) {
      error = commandErrorMessage(cause);
    } finally {
      busy = false;
    }
  }

  async function importArchive() {
    const path = await open({ multiple: false, filters: [{ name: "Bookshelf archive", extensions: ["age"] }] });
    if (!path) return;
    busy = true;
    error = "";
    try {
      library = await importLibraryArchive(path, archivePassword);
      feedback = "Личная библиотека восстановлена; вход в Codex потребуется выполнить заново";
    } catch (cause) {
      error = commandErrorMessage(cause);
    } finally {
      busy = false;
    }
  }

  async function exportDraft(draftId: string) {
    const path = await save({ defaultPath: "draft-note.md", filters: [{ name: "Markdown", extensions: ["md"] }] });
    if (!path) return;
    try {
      library = await exportDraftMarkdown(draftId, path);
      feedback = "Черновая заметка экспортирована и убрана из очереди";
    } catch (cause) {
      error = commandErrorMessage(cause);
    }
  }

  async function exportMaterial(materialId: string, title: string) {
    const path = await save({
      defaultPath: `${title || "material"}.md`,
      filters: [{ name: "Markdown", extensions: ["md"] }],
    });
    if (!path) return;
    try {
      await exportMaterialMarkdown(materialId, path);
      feedback = "Markdown-файл сохранён; его изменения не затронут Bookshelf";
    } catch (cause) {
      error = commandErrorMessage(cause);
    }
  }

  function navigate(target: View) {
    view = target;
    readingBook = null;
    settingsOpen = false;
    feedback = "";
    error = "";
  }
  async function completeSession(sessionId: string) {
    await run({ kind: "resolveSession", sessionId, status: "completed", reason: "" });
    const change = library?.lastDebtChange ?? 0;
    feedback = `Сеанс завершён. Долг ${change > 0 ? `вырос на ${change}` : change < 0 ? `уменьшился на ${Math.abs(change)}` : "не изменился"}`;
  }
  function bookTitle(bookId: string) {
    return library?.books.find((book) => book.id === bookId)?.title ?? "Книга";
  }
</script>

<svelte:head
  ><title>Bookshelf — личная библиотека</title><meta
    name="description"
    content="Локальное пространство для системного изучения технических книг"
  /></svelte:head
>

<WorkspaceShell>
  {#snippet sidebar()}
    <button
      class="flex items-center gap-3 border-0 bg-transparent text-lg text-white"
      aria-label="Открыть библиотеку"
      onclick={() => navigate("library")}
      ><span
        class="grid h-[42px] w-[35px] place-items-center rounded-[3px_9px_9px_3px] bg-[#b8d94a] font-display text-[23px] text-ink shadow-[inset_3px_0_#91ab36]"
        aria-hidden="true">B</span
      ><b>Bookshelf</b></button
    >
    <nav
      class="mt-3.5 flex gap-1 overflow-x-auto min-[901px]:mt-[46px] min-[901px]:grid"
      aria-label="Основная навигация"
    >
      <NavigationItem
        label="Библиотека"
        active={view === "library"}
        count={library?.books.length ?? 0}
        onclick={() => navigate("library")}
      />
      <NavigationItem
        label="Очередь разбора"
        active={view === "queue"}
        count={library?.drafts.length ?? 0}
        onclick={() => navigate("queue")}
      />
      <NavigationItem
        label="Идеи"
        active={view === "ideas"}
        count={library?.ideas.length ?? 0}
        onclick={() => navigate("ideas")}
      />
      <NavigationItem label="Изучение" active={view === "study"} onclick={() => navigate("study")} />
    </nav>
    <button
      class="absolute right-[18px] top-3.5 min-h-11 rounded-lg border-0 bg-transparent px-3 py-2.5 text-left text-spine-muted hover:bg-spine-raised hover:text-white min-[901px]:static min-[901px]:mt-auto"
      aria-expanded={settingsOpen}
      onclick={() => (settingsOpen = !settingsOpen)}>Настройки</button
    >
    <p class="mt-[18px] hidden px-2 text-[11px] leading-[1.4] text-[#aab2b9] min-[901px]:block">
      <i class="mr-1 inline-block size-[7px] rounded-full bg-[#b8d94a]"></i> Данные остаются на этом компьютере
    </p>
  {/snippet}
  {#if loading}
    <StatePanel live>Открываем личную библиотеку…</StatePanel>
  {:else if error && !library}
    <StatePanel tone="danger"
      ><strong>Библиотека не открылась</strong>
      <p>{error}</p></StatePanel
    >
  {:else if library}
    <PageHeader
      eyebrow="Личная библиотека"
      title={view === "library"
        ? "Рабочий стол читателя"
        : view === "queue"
          ? "Очередь разбора"
          : view === "ideas"
            ? "Идеи книги"
            : "Ритм изучения"}
    >
      {#snippet actions()}<Button variant="primary" onclick={importBook} disabled={busy}>Импортировать PDF</Button
        >{/snippet}
    </PageHeader>

    {#if error}<StatusMessage tone="danger">{error}</StatusMessage>{/if}
    {#if feedback}<div class="feedback" role="status">{feedback}</div>{/if}

    {#if settingsOpen}
      <section class="settings-panel" aria-label="Настройки">
        <div>
          <p class="eyebrow">Настройки</p>
          <h2>Личное напоминание</h2>
          <p>Короткая запись для себя — например, с чего начать в следующий раз.</p>
        </div>
        <form onsubmit={saveNote}>
          <TextField
            id="workspace-note"
            label="Личное напоминание"
            bind:value={note}
            placeholder="Например, вернуться к главе 2"
            maxlength={240}
            disabled={busy}
          />
          <div class="form-actions">
            <Button type="submit" disabled={busy}>Сохранить</Button>{#if saved}<StatusMessage tone="success"
                >Сохранено локально</StatusMessage
              >{/if}
          </div>
        </form>
        <div>
          <p class="eyebrow">Перенос и восстановление</p>
          <h2>Зашифрованный архив</h2>
          <p>
            Архив содержит рабочее состояние и PDF. Забытый пароль восстановить невозможно. Данные входа Codex не
            переносятся.
          </p>
        </div>
        <div>
          <TextField
            id="archive-password"
            label="Пароль архива"
            bind:value={archivePassword}
            placeholder="Не менее 8 символов"
            disabled={busy}
            type="password"
          />
          <div class="form-actions">
            <Button onclick={exportArchive} disabled={busy || archivePassword.length < 8}>Экспортировать</Button><Button
              onclick={importArchive}
              disabled={busy || archivePassword.length < 8}>Импортировать</Button
            ><Button
              onclick={async () => {
                try {
                  library = await restoreLatestSnapshot();
                  feedback = "Последний снимок восстановлен";
                } catch (cause) {
                  error = commandErrorMessage(cause);
                }
              }}>Восстановить снимок</Button
            >
          </div>
        </div>
        <div>
          <p class="eyebrow">Обновления</p>
          <h2>Совместимая версия Bookshelf</h2>
          <p>
            Устанавливаются только обновления с проверяемой подписью. При ошибке текущая версия и личная библиотека
            остаются без изменений.
          </p>
        </div>
        <div class="form-actions">
          <Button
            onclick={async () => {
              try {
                const installed = await installSignedUpdate();
                feedback = installed ? "Подписанное обновление установлено" : "У вас актуальная версия";
              } catch (cause) {
                error = commandErrorMessage(cause);
              }
            }}>Проверить обновления</Button
          >
        </div>
        <div>
          <p class="eyebrow">Ненавязчивое напоминание</p>
          <h2>Долг изучения</h2>
          <p>
            Одно системное уведомление появится только вне чтения, если объём долга не менялся выбранное число дней.
          </p>
        </div>
        <div class="form-actions">
          <label for="debt-days">Период без изменений</label><input
            id="debt-days"
            aria-label="Дней без изменения долга"
            type="number"
            min="1"
            max="90"
            value={library.debtReminderDays || 7}
            onchange={(event) =>
              run({ kind: "setDebtReminder", days: Number(event.currentTarget.value) }, "Период напоминания сохранён")}
          />
        </div>
        <div>
          <p class="eyebrow">Необязательная проверка</p>
          <h2>Вход в Codex</h2>
          <p>Codex хранит вход в отдельном каталоге. Bookshelf не читает OAuth-токены и не переносит их в архив.</p>
        </div>
        <div>
          <div class="form-actions">
            <Button disabled={codexLoginRunning} onclick={loginCodex}
              >{codexLoginRunning ? "Ожидаем вход…" : "Войти через ChatGPT"}</Button
            >{#if codexLoginUrl}<Button onclick={() => openUrl(codexLoginUrl)}>Открыть страницу входа</Button><strong
                aria-live="polite">Код: {codexLoginCode}</strong
              >{/if}
          </div>
        </div>
      </section>
    {/if}

    {#if view === "library"}
      <section class="bookmark" aria-label="Следующий шаг">
        <div>
          <span>Следующий шаг</span><strong>{nextStep}</strong>
          <p>Рекомендация учитывает доступную работу, но выбор всегда остаётся за вами.</p>
        </div>
        {#if activeBook}<Button onclick={() => openBook(activeBook)}>Продолжить чтение</Button
          >{:else if library.books.length === 0}<Button onclick={importBook}>Выбрать PDF</Button>{/if}
        <div class="debt"><small>Долг изучения</small><b>{debt}</b></div>
      </section>

      <form
        class="search"
        onsubmit={(event) => {
          event.preventDefault();
          search();
        }}
      >
        <label for="search">Поиск по книгам, идеям, темам, источникам и материалам</label>
        <div>
          <input id="search" bind:value={searchQuery} placeholder="Название или формулировка" /><Button type="submit"
            >Найти</Button
          >
        </div>
      </form>
      {#if searchQuery && results.length === 0}<p class="muted">Совпадений пока нет.</p>{/if}
      {#if results.length}<ul class="-mt-3 mb-[26px] grid list-none gap-px p-0">
          <!-- eslint-disable-next-line svelte/require-each-key -- TODO(ticket 13): key legacy search results in the search slice. -->
          {#each results as result}<li
              class="grid grid-cols-[62px_minmax(130px,.7fr)_minmax(180px,1fr)_auto] items-center gap-3 border-b border-rule px-1 py-3 max-[640px]:grid-cols-1"
            >
              <small class="uppercase">{searchResultKind(result.kind)}</small><b>{result.title}</b><span
                class="text-ink-muted">{result.context}</span
              ><Button onclick={() => openSearchResult(result)}>Открыть {result.title}</Button>
            </li>{/each}
        </ul>{/if}
      {#if readingBook}
        <section class="reader">
          <header>
            <div>
              <p class="eyebrow">Чтение · страница {readingBook.reading?.page ?? 1}</p>
              <h2>{readingBook.title}</h2>
            </div>
            <Button onclick={() => (readingBook = null)}>Закрыть</Button>
          </header>
          <PdfReader
            url={bookUrl}
            savedOutline={readingBook.outline}
            initialPage={readingBook.reading.page}
            initialZoom={readingBook.reading.zoom}
            initialScroll={readingBook.reading.scroll}
            onPosition={recordPosition}
            onOutline={acceptImportedOutline}
            onSelection={(selected, nearby) => {
              excerpt = selected;
              context = nearby;
            }}
          />
          <div class="reader-tools">
            <div>
              <h3>Место чтения</h3>
              <p>Bookshelf автоматически сохраняет страницу, масштаб и позицию.</p>
              <label for="reading-page">Страница</label><input
                id="reading-page"
                type="number"
                min="1"
                bind:value={readingBook.reading.page}
              /><label for="reading-zoom">Масштаб</label><input
                id="reading-zoom"
                type="number"
                min="0.5"
                max="4"
                step="0.1"
                bind:value={readingBook.reading.zoom}
              /><Button onclick={savePosition}>Сохранить сейчас</Button>
              <Button
                onclick={() =>
                  run(
                    { kind: "completeReading", bookId: readingBook!.id },
                    "Чтение завершено; изучение остаётся активным",
                  )}>Завершить чтение</Button
              >
              <details>
                <summary>Исправить оглавление</summary
                ><!-- eslint-disable-next-line svelte/require-each-key -- TODO(ticket 09): key legacy outline rows in the reading slice. -->
                {#each readingBook.outline ?? [] as item, index}<div
                    class="my-2 grid grid-cols-[minmax(130px,1fr)_90px_minmax(130px,.8fr)_auto_auto] gap-1.5 max-[640px]:grid-cols-1"
                  >
                    <input aria-label="Название раздела" bind:value={item.title} /><input
                      aria-label="Страница раздела"
                      type="number"
                      min="1"
                      bind:value={item.page}
                    /><select aria-label="Родительский раздел" bind:value={item.parentId}
                      ><option value={null}>Без родителя</option
                      ><!-- eslint-disable-next-line svelte/require-each-key -- TODO(ticket 09): key legacy outline parent options in the reading slice. -->
                      {#each readingBook.outline.filter((candidate) => candidate.id !== item.id) as parent}<option
                          value={parent.id}>{parent.title}</option
                        >{/each}</select
                    ><Button
                      disabled={index === 0}
                      onclick={() => {
                        const list = readingBook!.outline;
                        [list[index - 1], list[index]] = [list[index], list[index - 1]];
                        saveOutline();
                      }}>Выше</Button
                    ><Button
                      onclick={() => {
                        readingBook!.outline = readingBook!.outline
                          .filter((candidate) => candidate.id !== item.id)
                          .map((candidate) =>
                            candidate.parentId === item.id ? { ...candidate, parentId: null } : candidate,
                          );
                        saveOutline();
                      }}>Удалить</Button
                    >
                  </div>{/each}
                <form
                  class="my-2 grid grid-cols-[minmax(130px,1fr)_90px_auto] gap-1.5 max-[640px]:grid-cols-1"
                  onsubmit={addOutlineItem}
                >
                  <input aria-label="Новый раздел" bind:value={outlineTitle} placeholder="Название главы" /><input
                    aria-label="Страница нового раздела"
                    type="number"
                    min="1"
                    bind:value={outlinePage}
                  /><Button type="submit">Добавить</Button>
                </form>
                <Button onclick={saveOutline}>Сохранить правки</Button>
              </details>
            </div>
            <form onsubmit={captureDraft}>
              <h3>Черновая заметка</h3>
              <label for="section">Глава или раздел</label><input id="section" bind:value={section} /><label
                for="excerpt">Выделенный фрагмент</label
              ><textarea id="excerpt" bind:value={excerpt} required></textarea><label for="context"
                >Непосредственный контекст</label
              ><textarea id="context" bind:value={context}></textarea><label for="comment"
                >Моя мысль (необязательно)</label
              ><textarea id="comment" bind:value={comment}></textarea><Button type="submit">В очередь разбора</Button>
            </form>
          </div>
        </section>
      {:else if library.books.length === 0}
        <section class="empty">
          <div aria-hidden="true">▥</div>
          <p class="eyebrow">Ваша первая закладка</p>
          <h2>Начните с одной важной книги</h2>
          <p>
            Импортируйте PDF с текстовым слоем. Bookshelf сделает локальную копию, сохранит место чтения и поможет
            превратить фрагменты в применимые идеи.
          </p>
          <Button variant="primary" onclick={importBook}>Импортировать первую книгу</Button><small
            >PDF не покинет этот компьютер</small
          >
        </section>
      {:else}
        <section class="books" aria-label="Книги">
          <!-- eslint-disable-next-line svelte/require-each-key -- TODO(ticket 09): key legacy book cards in the reading slice. -->
          {#each library.books as book}<article>
              <div class="cover" aria-hidden="true"><span>{book.title.slice(0, 1)}</span></div>
              <div>
                <p class="eyebrow">
                  {book.readingCompleted ? "Чтение завершено" : `Страница ${book.reading?.page ?? 1}`}
                </p>
                <h2>{book.title}</h2>
                <p>
                  {book.id === library.activeStudyBookId
                    ? "Активное изучение"
                    : book.studyCompleted
                      ? "Изучение завершено"
                      : "Готова к изучению"}
                </p>
                <div class="card-actions">
                  <Button onclick={() => openBook(book)}>{book.reading?.page > 1 ? "Продолжить" : "Открыть"}</Button
                  >{#if book.id !== library.activeStudyBookId && !book.studyCompleted}<Button
                      onclick={() =>
                        run({ kind: "activateStudy", bookId: book.id }, "Книга выбрана для активного изучения")}
                      >Изучать</Button
                    >{/if}
                </div>
              </div>
            </article>{/each}
        </section>
      {/if}
    {:else if view === "queue"}
      {#if library.drafts.length === 0}<section class="empty compact">
          <h2>Очередь разобрана</h2>
          <p>Новые фрагменты можно сохранить, не выходя из просмотрщика.</p>
        </section>{:else}<section class="stack">
          <!-- eslint-disable-next-line svelte/require-each-key -- TODO(ticket 08): key legacy draft cards in the draft lifecycle slice. -->
          {#each library.drafts as draft}<article class="work-card">
              <p class="eyebrow">{bookTitle(draft.bookId)} · {draft.section} · стр. {draft.page}</p>
              <blockquote>{draft.excerpt}</blockquote>
              {#if draft.comment}<p>{draft.comment}</p>{/if}<label for={`idea-${draft.id}`}
                >Самостоятельная формулировка</label
              ><textarea id={`idea-${draft.id}`} bind:value={formulation}></textarea>
              <div class="card-actions">
                <Button variant="primary" onclick={() => resolveDraft(draft.id)}>Создать идею</Button><select
                  aria-label="Идея для присоединения"
                  bind:value={attachIdeaId}
                  ><option value="">Выберите идею</option
                  ><!-- eslint-disable-next-line svelte/require-each-key -- TODO(ticket 08): key legacy draft target options in the draft lifecycle slice. -->
                  {#each library.ideas as idea}<option value={idea.id}>{idea.formulation}</option>{/each}</select
                ><Button
                  disabled={!attachIdeaId}
                  onclick={() =>
                    run(
                      { kind: "attachDraftToIdea", draftId: draft.id, ideaId: attachIdeaId },
                      "Фрагмент присоединён к идее",
                    )}>Присоединить</Button
                ><Button onclick={() => exportDraft(draft.id)}>Экспортировать</Button><Button
                  onclick={() => run({ kind: "discardDraft", draftId: draft.id }, "Черновая заметка удалена")}
                  >Удалить</Button
                >
              </div>
            </article>{/each}
        </section>{/if}
    {:else if view === "ideas"}
      {#if openedIdea}
        <section
          class="mb-5 rounded-[11px] border border-rule bg-paper-raised p-5 shadow-paper"
          aria-label="Открытая запись поиска"
        >
          <p class="eyebrow">Идея · {bookTitle(openedIdea.bookId)} · {openedIdea.section}</p>
          <h2>{openedIdea.formulation}</h2>
          <!-- eslint-disable-next-line svelte/require-each-key -- TODO(ticket 10): key legacy source fragments in the idea workflow slice. -->
          {#each openedIdea.fragments as fragment}<blockquote>
              стр. {fragment.page}: {fragment.excerpt}{#if fragment.context}<br /><small>{fragment.context}</small>{/if}
            </blockquote>{/each}<Button
            onclick={() => {
              openedSearchResult = null;
              navigate("library");
            }}>Вернуться к поиску</Button
          >
        </section>
      {:else if openedTopic}
        <section
          class="mb-5 rounded-[11px] border border-rule bg-paper-raised p-5 shadow-paper"
          aria-label="Открытая запись поиска"
        >
          <p class="eyebrow">Тема знаний</p>
          <h2>{openedTopic.name}</h2>
          <!-- eslint-disable-next-line svelte/require-each-key -- TODO(ticket 10): key legacy topic ideas in the idea workflow slice. -->
          {#each library.ideas.filter((idea) => idea.topicIds.includes(openedTopic!.id)) as idea}<article
              class="border-t border-rule py-3"
            >
              <b>{idea.formulation}</b>
              <p class="mb-0 text-ink-muted">{bookTitle(idea.bookId)} · {idea.section}</p>
            </article>{/each}<Button
            onclick={() => {
              openedSearchResult = null;
              navigate("library");
            }}>Вернуться к поиску</Button
          >
        </section>
      {:else if openedMaterial}
        <section
          class="mb-5 rounded-[11px] border border-rule bg-paper-raised p-5 shadow-paper"
          aria-label="Открытая запись поиска"
        >
          <p class="eyebrow">Материал для передачи</p>
          <h2>{openedMaterial.title}</h2>
          <p><b>Проблема:</b> {openedMaterial.problem}</p>
          <p><b>Идея:</b> {openedMaterial.idea}</p>
          <p><b>Пример:</b> {openedMaterial.example}</p>
          <p><b>Результат:</b> {openedMaterial.result}</p>
          <p><b>Ограничения:</b> {openedMaterial.limitations}</p>
          <Button
            onclick={() => {
              openedSearchResult = null;
              navigate("library");
            }}>Вернуться к поиску</Button
          >
        </section>
      {/if}
      <section class="section-head">
        <div>
          <h2>Темы знаний</h2>
          <p>Темы и связи появляются только после вашего подтверждения.</p>
        </div>
        <form
          onsubmit={(event) => {
            event.preventDefault();
            run({ kind: "createTopic", name: topicName }, "Тема создана");
            topicName = "";
          }}
        >
          <input aria-label="Название темы" bind:value={topicName} placeholder="Например, архитектура данных" /><Button
            type="submit">Создать тему</Button
          >
        </form>
      </section>
      {#if library.ideas.length === 0}<section class="empty compact">
          <h2>Здесь появятся ваши идеи</h2>
          <p>Разберите черновую заметку, чтобы сохранить авторскую формулировку и источник.</p>
        </section>{:else}<IdeaWorkbench
          {library}
          {run}
          {bookTitle}
          onLibrary={(next) => {
            library = next;
          }}
        />{#if library.materials.length}<section class="stack review-packages" aria-label="Материалы для передачи">
            <!-- eslint-disable-next-line svelte/require-each-key -- TODO(ticket 13): key legacy material cards in the export slice. -->
            {#each library.materials as material}<article class="work-card">
                <p class="eyebrow">Материал для передачи</p>
                <h2>{material.title}</h2>
                <div class="card-actions">
                  <Button
                    onclick={async () => {
                      await navigator.clipboard.writeText(`# ${material.title}\n\n${material.idea}`);
                      feedback = "Материал скопирован";
                    }}>Скопировать</Button
                  ><Button onclick={() => exportMaterial(material.id, material.title)}>Сохранить Markdown</Button>
                </div>
              </article>{/each}
          </section>{/if}{/if}
    {:else}
      <section class="study-grid">
        <article class="work-card">
          <p class="eyebrow">Недельный ритм</p>
          <h2>{library.weeklySessionBudget || 3} сеанса</h2>
          <p>Без дедлайна книги, нормы страниц и штрафов.</p>
          <div class="rhythm">
            <!-- eslint-disable-next-line svelte/require-each-key -- TODO(ticket 11): key legacy recall cadence controls in the recall slice. -->
            {#each [2, 3, 4, 5] as count}<button
                class:chosen={library.weeklySessionBudget === count}
                onclick={() => run({ kind: "setStudyRhythm", weeklySessionBudget: count }, "Недельный ритм сохранён")}
                >{count}</button
              >{/each}
          </div>
        </article>
        <article class="work-card">
          <p class="eyebrow">Новый сеанс</p>
          <h2>С каким намерением?</h2>
          <form
            onsubmit={(event) => {
              event.preventDefault();
              run(
                { kind: "planSession", intention: sessionIntention, plannedAt: Math.floor(Date.now() / 1000) },
                "Сеанс запланирован",
              );
              sessionIntention = "";
            }}
          >
            <textarea
              aria-label="Намерение сеанса"
              bind:value={sessionIntention}
              placeholder="Разобрать две заметки и продолжить главу"
              required></textarea><Button type="submit">Запланировать</Button>
          </form>
        </article>
      </section>
      {#if library.sessions.length}<section class="stack">
          <!-- eslint-disable-next-line svelte/require-each-key -- TODO(ticket 12): key legacy session cards in the study session slice. -->
          {#each library.sessions as session}<article class="session">
              <div>
                <b>{session.intention}</b><small
                  >{session.status === "planned"
                    ? "Запланирован"
                    : session.status === "active"
                      ? "Сеанс идёт"
                      : session.status}</small
                >
              </div>
              {#if session.status === "planned"}<div class="session-actions">
                  <Button
                    variant="primary"
                    onclick={() =>
                      run(
                        { kind: "startSession", sessionId: session.id },
                        "Сеанс начат; изменение долга будет измерено отсюда",
                      )}>Начать сеанс</Button
                  ><select aria-label="Решение по пропущенному сеансу" bind:value={missedSessionStatus}
                    ><option value="moved">Перенести</option><option value="replaced">Заменить</option><option
                      value="cancelled">Отменить</option
                    ></select
                  ><input
                    aria-label="Причина решения"
                    bind:value={missedSessionReason}
                    placeholder="Почему план изменился"
                  /><Button
                    onclick={() =>
                      run(
                        {
                          kind: "resolveSession",
                          sessionId: session.id,
                          status: missedSessionStatus,
                          reason: missedSessionReason,
                        },
                        "Решение по сеансу сохранено без штрафа",
                      )}>Сохранить решение</Button
                  >
                </div>{:else if session.status === "active"}<Button onclick={() => completeSession(session.id)}
                  >Завершить сеанс</Button
                >{/if}
            </article>{/each}
        </section>{/if}
      {#if activeBook}<form
          class="work-card retrospective"
          onsubmit={(event) => {
            event.preventDefault();
            run(
              {
                kind: "completeStudy",
                bookId: activeBook!.id,
                retrospective,
                significantIdeaIds,
                continuingWork,
                debtDecision,
              },
              "Изучение завершено; продолжающаяся работа осталась доступна",
            );
          }}
        >
          <p class="eyebrow">Явное завершение изучения</p>
          <h2>Ретроспектива «{activeBook.title}»</h2>
          <label for="retrospective">Результаты применения и изменения в понимании или действиях</label><textarea
            id="retrospective"
            bind:value={retrospective}></textarea>
          <fieldset>
            <legend>3–7 значимых идей</legend
            ><!-- eslint-disable-next-line svelte/require-each-key -- TODO(ticket 12): key legacy retrospective idea controls in the study completion slice. -->
            {#each library.ideas.filter((idea) => idea.bookId === activeBook?.id) as idea}<label class="checkbox"
                ><input type="checkbox" value={idea.id} bind:group={significantIdeaIds} /> {idea.formulation}</label
              >{/each}
          </fieldset>
          <label for="continuing">Продолжающиеся эксперименты или восстановления</label><textarea
            id="continuing"
            bind:value={continuingWork}></textarea><label for="debt-decision">Решение по оставшемуся долгу</label
          ><textarea id="debt-decision" bind:value={debtDecision}></textarea><Button variant="primary" type="submit"
            >Завершить изучение</Button
          >
        </form>{/if}
    {/if}
  {/if}
</WorkspaceShell>

<style>
  h2 {
    margin: 0 0 8px;
    font-family: Georgia, serif;
    font-size: 25px;
    font-weight: 500;
    line-height: 1.15;
  }
  h3,
  p {
    margin-top: 0;
  }
  .eyebrow {
    margin: 0 0 7px;
    color: #66717a;
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.11em;
    text-transform: uppercase;
  }
  .feedback {
    min-height: 24px;
    margin: -8px 0 18px;
    color: var(--color-success);
    font-size: 13px;
  }
  .settings-panel {
    display: grid;
    grid-template-columns: 0.8fr 1.2fr;
    gap: 34px;
    margin-bottom: 24px;
    border: 1px solid var(--color-rule);
    border-radius: 12px;
    background: var(--color-paper-raised);
    padding: 24px;
    box-shadow: var(--shadow-paper);
  }
  .settings-panel p,
  .muted {
    color: var(--color-ink-muted);
    line-height: 1.55;
  }
  .form-actions,
  .card-actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 9px;
    margin-top: 10px;
  }
  .bookmark {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto 92px;
    align-items: center;
    gap: 24px;
    overflow: hidden;
    margin-bottom: 30px;
    border-radius: 4px 13px 13px 4px;
    background: #eaf1c8;
    padding: 22px 26px 22px 34px;
    box-shadow: inset 7px 0 #b8d94a;
  }
  .bookmark span,
  .debt small {
    display: block;
    margin-bottom: 5px;
    color: #53602d;
    font-size: 10px;
    font-weight: 800;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }
  .bookmark strong {
    font-family: Georgia, serif;
    font-size: 22px;
    font-weight: 500;
  }
  .bookmark p {
    margin: 6px 0 0;
    color: #606a50;
    font-size: 13px;
  }
  .debt {
    align-self: stretch;
    border-left: 1px solid #ccd69e;
    padding-left: 22px;
    text-align: center;
  }
  .debt b {
    font:
      34px ui-monospace,
      monospace;
  }
  .search {
    margin-bottom: 24px;
  }
  .search label,
  :global(form label) {
    display: block;
    margin-bottom: 6px;
    color: #4d5861;
    font-size: 12px;
    font-weight: 700;
  }
  .search > div,
  .section-head form {
    display: flex;
    gap: 8px;
  }
  input,
  textarea,
  select {
    width: 100%;
    min-width: 0;
    border: 1px solid #cfd1cd;
    border-radius: 8px;
    background: #fffefa;
    padding: 11px 12px;
    color: var(--color-ink);
    outline: none;
  }
  textarea {
    min-height: 84px;
    resize: vertical;
  }
  input:focus,
  textarea:focus,
  select:focus {
    border-color: #697c39;
    box-shadow: 0 0 0 3px var(--color-focus);
  }
  .empty {
    display: grid;
    justify-items: center;
    border: 1px solid var(--color-rule);
    border-radius: 14px;
    background: var(--color-paper-raised);
    padding: 56px 28px;
    text-align: center;
    box-shadow: var(--shadow-paper);
  }
  .empty > div {
    display: grid;
    width: 54px;
    height: 64px;
    place-items: center;
    margin-bottom: 20px;
    border-radius: 4px 11px 11px 4px;
    background: #eaf1c8;
    box-shadow: inset 5px 0 #b8d94a;
    font-size: 26px;
  }
  .empty > p:not(.eyebrow) {
    max-width: 590px;
    color: var(--color-ink-muted);
    line-height: 1.65;
  }
  .empty > small {
    margin-top: 13px;
    color: var(--color-ink-muted);
  }
  .empty.compact {
    padding: 42px 24px;
  }
  .books,
  .stack {
    display: grid;
    gap: 14px;
  }
  .books article {
    display: grid;
    grid-template-columns: 104px 1fr;
    gap: 24px;
    border-bottom: 1px solid var(--color-rule);
    padding: 20px 0;
  }
  .cover {
    display: grid;
    min-height: 138px;
    place-items: center;
    border-radius: 3px 9px 9px 3px;
    background: #283847;
    color: #dce9a7;
    box-shadow:
      inset 5px 0 #101923,
      0 8px 17px #17212a24;
  }
  .cover span {
    font-family: Georgia, serif;
    font-size: 40px;
  }
  .books article > div:last-child {
    align-self: center;
  }
  .books article p {
    color: var(--color-ink-muted);
  }
  .reader,
  .work-card {
    border: 1px solid var(--color-rule);
    border-radius: 11px;
    background: var(--color-paper-raised);
    padding: 24px;
    box-shadow: var(--shadow-paper);
  }
  .reader > header,
  .section-head {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 24px;
    margin-bottom: 26px;
  }
  .reader-tools {
    display: grid;
    grid-template-columns: 0.7fr 1.3fr;
    gap: 28px;
    padding-top: 24px;
  }
  .reader-tools p {
    color: var(--color-ink-muted);
  }
  .reader-tools form,
  .retrospective {
    display: grid;
    gap: 7px;
  }
  blockquote {
    margin: 14px 0;
    border-left: 3px solid #b8d94a;
    padding: 6px 0 6px 16px;
    color: #45515a;
    line-height: 1.55;
  }
  details {
    margin-top: 14px;
    border-top: 1px solid var(--color-rule);
    padding-top: 12px;
  }
  summary {
    cursor: pointer;
    font-weight: 700;
  }
  .section-head {
    align-items: center;
  }
  .section-head p {
    color: var(--color-ink-muted);
  }
  .section-head form {
    width: min(460px, 50%);
  }
  .study-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    margin-bottom: 20px;
  }
  .rhythm {
    display: flex;
    gap: 7px;
  }
  .rhythm button {
    width: 42px;
    height: 42px;
    border: 1px solid var(--color-rule);
    border-radius: 50%;
    background: white;
  }
  .rhythm button.chosen {
    border-color: #72843d;
    background: #eaf1c8;
    font-weight: 800;
  }
  .session {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
    border-bottom: 1px solid var(--color-rule);
    padding: 15px 5px;
  }
  .session small {
    display: block;
    margin-top: 4px;
    color: var(--color-ink-muted);
  }
  .session-actions {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .session-actions select {
    width: auto;
  }
  .session-actions input {
    width: 210px;
  }
  .review-packages,
  .retrospective {
    margin-top: 20px;
  }
  .checkbox {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 7px 0;
  }
  .checkbox input {
    width: 18px;
    min-height: 18px;
  }
  fieldset {
    margin: 5px 0 10px;
    border: 1px solid var(--color-rule);
    border-radius: 8px;
    padding: 10px 13px;
  }
  legend {
    padding: 0 5px;
    color: #4d5861;
    font-size: 12px;
    font-weight: 700;
  }
  @media (max-width: 640px) {
    .bookmark {
      grid-template-columns: 1fr;
      padding: 20px 20px 20px 27px;
    }
    .debt {
      border-top: 1px solid #ccd69e;
      border-left: 0;
      padding: 13px 0 0;
      text-align: left;
    }
    .settings-panel,
    .reader-tools,
    .study-grid {
      grid-template-columns: 1fr;
    }
    .books article {
      grid-template-columns: 72px 1fr;
    }
    .cover {
      min-height: 104px;
    }
    .section-head,
    .session,
    .session-actions {
      align-items: stretch;
      flex-direction: column;
    }
    .section-head form,
    .session-actions input,
    .session-actions select {
      width: 100%;
    }
  }
</style>
