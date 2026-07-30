<script lang="ts">
  import { onMount } from "svelte";
  import {
    Button,
    CheckboxField,
    NavigationItem,
    NumberField,
    PageHeader,
    SelectField,
    StatePanel,
    StatusMessage,
    TextField,
    TextArea,
    WorkspaceShell,
  } from "@/shared/ui";
  import IdeaWorkbench from "./IdeaWorkbench.svelte";
  import PdfReader from "./PdfReader.svelte";
  import {
    commandErrorMessage,
    type Book,
    type LibraryAction,
    type SessionStatus,
    type SearchResult,
  } from "@/shared/api";
  import { tauriLibraryCommands } from "../api/tauri-library-commands";
  import type { LibraryCommands, LibraryView } from "../model/library-commands";
  import { LibrarySession } from "../model/library-session.svelte";

  let { commands = tauriLibraryCommands }: { commands?: LibraryCommands } = $props();
  const session = new LibrarySession(() => commands);
  let library = $derived(session.library);
  let view = $derived(session.view);
  let loading = $derived(session.loading);
  let busy = $derived(session.busy);
  let error = $derived(session.error);
  let feedback = $derived(session.feedback);
  let note = $state("");
  let saved = $state(false);
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
  let archivePassword = $state("");
  let rescheduledSessionStatus = $state<SessionStatus>("moved");
  let rescheduledSessionReason = $state("");
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

  onMount(() => {
    let stop: (() => void) | undefined;
    void commands
      .onCodexLogin((event) => {
        if (event.kind !== "deviceCode") return;
        const [loginUrl = "", loginCode = ""] = event.text.split("\n", 2);
        codexLoginUrl = loginUrl;
        codexLoginCode = loginCode;
      })
      .then((unlisten) => (stop = unlisten));
    void session.load().then(() => {
      note = session.library?.workspaceNote ?? "";
    });
    return () => stop?.();
  });

  async function loginCodex() {
    codexLoginRunning = true;
    session.error = "";
    codexLoginUrl = "";
    codexLoginCode = "";
    try {
      await commands.startCodexLogin();
      session.feedback = "Вход в Codex завершён";
    } catch (cause) {
      session.error = commandErrorMessage(cause);
    } finally {
      codexLoginRunning = false;
    }
  }

  async function run(action: LibraryAction, success = "Изменения сохранены") {
    await session.execute(action, success);
  }

  async function importBook() {
    session.error = "";
    session.busy = true;
    const order = session.beginSnapshotRequest();
    try {
      const snapshot = await commands.importPdf();
      if (!snapshot) return;
      session.replaceFrom(snapshot, order);
      session.feedback = "Книга скопирована в личную библиотеку";
      session.view = "library";
    } catch (cause) {
      session.error = commandErrorMessage(cause);
    } finally {
      session.busy = false;
    }
  }

  async function openBook(book: Book) {
    session.busy = true;
    session.error = "";
    try {
      bookUrl = await commands.bookUrl(book.id);
      readingBook = book;
    } catch (cause) {
      session.error = commandErrorMessage(cause);
    } finally {
      session.busy = false;
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
      await session.execute({ kind: "updateReading", bookId, page, zoom, scroll }, "Место чтения сохранено");
      readingBook = session.library?.books.find((book) => book.id === bookId) ?? null;
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
    if (!session.error) {
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
    if (!session.error) formulation = "";
  }

  async function saveNote(event: SubmitEvent) {
    event.preventDefault();
    saved = false;
    saved = await session.execute({ kind: "saveWorkspaceNote", note }, "Сохранено локально");
  }

  async function search() {
    openedSearchResult = null;
    try {
      results = await commands.search(searchQuery);
    } catch (cause) {
      session.error = commandErrorMessage(cause);
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

  function sessionStatusLabel(status: SessionStatus) {
    return (
      {
        planned: "Запланирован",
        active: "Сеанс идёт",
        completed: "Завершён",
        moved: "Перенесён",
        replaced: "Заменён",
        cancelled: "Отменён",
      } satisfies Record<SessionStatus, string>
    )[status];
  }

  async function exportArchive() {
    session.busy = true;
    session.error = "";
    try {
      if (await commands.exportArchive(archivePassword)) session.feedback = "Зашифрованный архив сохранён";
    } catch (cause) {
      session.error = commandErrorMessage(cause);
    } finally {
      session.busy = false;
    }
  }

  async function importArchive() {
    session.busy = true;
    session.error = "";
    const order = session.beginSnapshotRequest();
    try {
      const snapshot = await commands.importArchive(archivePassword);
      if (!snapshot) return;
      session.replaceFrom(snapshot, order);
      session.feedback = "Личная библиотека восстановлена; вход в Codex потребуется выполнить заново";
    } catch (cause) {
      session.error = commandErrorMessage(cause);
    } finally {
      session.busy = false;
    }
  }

  async function exportDraft(draftId: string) {
    try {
      const snapshot = await commands.exportDraft(draftId);
      if (!snapshot) return;
      session.replaceFrom(snapshot);
      session.feedback = "Черновая заметка экспортирована и убрана из очереди";
    } catch (cause) {
      session.error = commandErrorMessage(cause);
    }
  }

  async function exportMaterial(materialId: string, title: string) {
    try {
      if (await commands.exportMaterial(materialId, title))
        session.feedback = "Markdown-файл сохранён; его изменения не затронут Bookshelf";
    } catch (cause) {
      session.error = commandErrorMessage(cause);
    }
  }

  function navigate(target: LibraryView) {
    session.navigate(target);
    readingBook = null;
    settingsOpen = false;
  }
  async function completeSession(sessionId: string) {
    await run({ kind: "resolveSession", sessionId, status: "completed", reason: "" });
    const change = library?.lastDebtChange ?? 0;
    session.feedback = `Сеанс завершён. Долг ${change > 0 ? `вырос на ${change}` : change < 0 ? `уменьшился на ${Math.abs(change)}` : "не изменился"}`;
  }
  function bookTitle(bookId: string) {
    return library?.books.find((book) => book.id === bookId)?.title ?? "Книга";
  }
  function setSignificantIdea(ideaId: string, checked: boolean) {
    significantIdeaIds = checked
      ? [...new Set([...significantIdeaIds, ideaId])]
      : significantIdeaIds.filter((id) => id !== ideaId);
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
  <div
    class="[&_h2]:mb-2 [&_h2]:font-display [&_h2]:text-[25px] [&_h2]:font-medium [&_h2]:leading-[1.15] [&_h3]:mt-0 [&_p]:mt-0 [&_form_label]:mb-1.5 [&_form_label]:block [&_form_label]:text-xs [&_form_label]:font-bold [&_form_label]:text-[#4d5861] [&_input]:w-full [&_input]:min-w-0 [&_input]:rounded-lg [&_input]:border [&_input]:border-[#cfd1cd] [&_input]:bg-paper-raised [&_input]:px-3 [&_input]:py-[11px] [&_input]:text-ink [&_input]:outline-none [&_textarea]:min-h-[84px] [&_textarea]:w-full [&_textarea]:min-w-0 [&_textarea]:resize-y [&_textarea]:rounded-lg [&_textarea]:border [&_textarea]:border-[#cfd1cd] [&_textarea]:bg-paper-raised [&_textarea]:px-3 [&_textarea]:py-[11px] [&_textarea]:text-ink [&_textarea]:outline-none [&_input:focus]:border-leaf [&_input:focus]:ring-3 [&_input:focus]:ring-focus [&_textarea:focus]:border-leaf [&_textarea:focus]:ring-3 [&_textarea:focus]:ring-focus [&_blockquote]:my-3.5 [&_blockquote]:border-l-[3px] [&_blockquote]:border-[#b8d94a] [&_blockquote]:py-1.5 [&_blockquote]:pl-4 [&_blockquote]:leading-[1.55] [&_blockquote]:text-[#45515a] [&_details]:mt-3.5 [&_details]:border-t [&_details]:border-rule [&_details]:pt-3 [&_summary]:cursor-pointer [&_summary]:font-bold [&_fieldset]:mb-2.5 [&_fieldset]:mt-[5px] [&_fieldset]:rounded-lg [&_fieldset]:border [&_fieldset]:border-rule [&_fieldset]:px-[13px] [&_fieldset]:py-2.5 [&_legend]:px-[5px] [&_legend]:text-xs [&_legend]:font-bold [&_legend]:text-[#4d5861]"
  >
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
      {#if feedback}<div class="-mt-2 mb-[18px] min-h-6 text-[13px] text-success" role="status">{feedback}</div>{/if}

      {#if settingsOpen}
        <section
          class="mb-6 grid grid-cols-[.8fr_1.2fr] gap-[34px] rounded-xl border border-rule bg-paper-raised p-6 shadow-paper max-[640px]:grid-cols-1 [&_p]:leading-[1.55] [&_p]:text-ink-muted"
          aria-label="Настройки"
        >
          <div>
            <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">Настройки</p>
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
            <div class="mt-2.5 flex flex-wrap items-center gap-[9px]">
              <Button type="submit" disabled={busy}>Сохранить</Button>{#if saved}<StatusMessage tone="success"
                  >Сохранено локально</StatusMessage
                >{/if}
            </div>
          </form>
          <div>
            <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">
              Перенос и восстановление
            </p>
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
            <div class="mt-2.5 flex flex-wrap items-center gap-[9px]">
              <Button onclick={exportArchive} disabled={busy || archivePassword.length < 8}>Экспортировать</Button
              ><Button onclick={importArchive} disabled={busy || archivePassword.length < 8}>Импортировать</Button
              ><Button
                onclick={async () => {
                  try {
                    session.replaceFrom(await commands.restoreLatestSnapshot());
                    session.feedback = "Последний снимок восстановлен";
                  } catch (cause) {
                    session.error = commandErrorMessage(cause);
                  }
                }}>Восстановить снимок</Button
              >
            </div>
          </div>
          <div>
            <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">Обновления</p>
            <h2>Совместимая версия Bookshelf</h2>
            <p>
              Устанавливаются только обновления с проверяемой подписью. При ошибке текущая версия и личная библиотека
              остаются без изменений.
            </p>
          </div>
          <div class="mt-2.5 flex flex-wrap items-center gap-[9px]">
            <Button
              onclick={async () => {
                try {
                  const installed = await commands.installSignedUpdate();
                  session.feedback = installed ? "Подписанное обновление установлено" : "У вас актуальная версия";
                } catch (cause) {
                  session.error = commandErrorMessage(cause);
                }
              }}>Проверить обновления</Button
            >
          </div>
          <div>
            <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">
              Ненавязчивое напоминание
            </p>
            <h2>Долг изучения</h2>
            <p>
              Одно системное уведомление появится только вне чтения, если объём долга не менялся выбранное число дней.
            </p>
          </div>
          <div class="mt-2.5 flex flex-wrap items-center gap-[9px]">
            <NumberField
              id="debt-days"
              label="Период без изменений"
              ariaLabel="Дней без изменения долга"
              min={1}
              max={90}
              value={library.debtReminderDays || 7}
              onChange={(days) => run({ kind: "setDebtReminder", days }, "Период напоминания сохранён")}
            />
          </div>
          <div>
            <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">
              Необязательная проверка
            </p>
            <h2>Вход в Codex</h2>
            <p>Codex хранит вход в отдельном каталоге. Bookshelf не читает OAuth-токены и не переносит их в архив.</p>
          </div>
          <div>
            <div class="mt-2.5 flex flex-wrap items-center gap-[9px]">
              <Button disabled={codexLoginRunning} onclick={loginCodex}
                >{codexLoginRunning ? "Ожидаем вход…" : "Войти через ChatGPT"}</Button
              >{#if codexLoginUrl}<Button onclick={() => commands.openExternalUrl(codexLoginUrl)}
                  >Открыть страницу входа</Button
                ><strong aria-live="polite">Код: {codexLoginCode}</strong>{/if}
            </div>
          </div>
        </section>
      {/if}

      {#if view === "library"}
        <section
          class="mb-[30px] grid grid-cols-[minmax(0,1fr)_auto_92px] items-center gap-6 overflow-hidden rounded-[4px_13px_13px_4px] bg-[#eaf1c8] py-[22px] pl-[34px] pr-[26px] shadow-[inset_7px_0_#b8d94a] max-[640px]:grid-cols-1 max-[640px]:py-5 max-[640px]:pl-[27px] max-[640px]:pr-5 [&_span]:mb-[5px] [&_span]:block [&_span]:text-[10px] [&_span]:font-extrabold [&_span]:uppercase [&_span]:tracking-[.1em] [&_span]:text-[#53602d] [&_strong]:font-display [&_strong]:text-[22px] [&_strong]:font-medium [&_p]:mb-0 [&_p]:mt-1.5 [&_p]:text-[13px] [&_p]:text-[#606a50]"
          aria-label="Следующий шаг"
        >
          <div>
            <span>Следующий шаг</span><strong>{nextStep}</strong>
            <p>Рекомендация учитывает доступную работу, но выбор всегда остаётся за вами.</p>
          </div>
          {#if activeBook}<Button onclick={() => openBook(activeBook)}>Продолжить чтение</Button
            >{:else if library.books.length === 0}<Button onclick={importBook}>Выбрать PDF</Button>{/if}
          <div
            class="self-stretch border-l border-[#ccd69e] pl-[22px] text-center max-[640px]:border-l-0 max-[640px]:border-t max-[640px]:pb-0 max-[640px]:pl-0 max-[640px]:pt-[13px] max-[640px]:text-left [&_small]:mb-[5px] [&_small]:block [&_small]:text-[10px] [&_small]:font-extrabold [&_small]:uppercase [&_small]:tracking-[.1em] [&_small]:text-[#53602d] [&_b]:font-mono [&_b]:text-[34px]"
          >
            <small>Долг изучения</small><b>{debt}</b>
          </div>
        </section>

        <form
          class="mb-6 [&>div]:flex [&>div]:gap-2 [&>label]:mb-1.5 [&>label]:block [&>label]:text-xs [&>label]:font-bold [&>label]:text-[#4d5861]"
          onsubmit={(event) => {
            event.preventDefault();
            search();
          }}
        >
          <label for="search">Поиск по книгам, идеям, темам, источникам и материалам</label>
          <div>
            <TextField id="search" bind:value={searchQuery} placeholder="Название или формулировка" />
            <Button type="submit">Найти</Button>
          </div>
        </form>
        {#if searchQuery && results.length === 0}<p class="leading-[1.55] text-ink-muted">Совпадений пока нет.</p>{/if}
        {#if results.length}<ul class="-mt-3 mb-[26px] grid list-none gap-px p-0">
            {#each results as result (`${result.kind}-${result.id}`)}<li
                class="grid grid-cols-[62px_minmax(130px,.7fr)_minmax(180px,1fr)_auto] items-center gap-3 border-b border-rule px-1 py-3 max-[640px]:grid-cols-1"
              >
                <small class="uppercase">{searchResultKind(result.kind)}</small><b>{result.title}</b><span
                  class="text-ink-muted">{result.context}</span
                ><Button onclick={() => openSearchResult(result)}>Открыть {result.title}</Button>
              </li>{/each}
          </ul>{/if}
        {#if readingBook}
          <section
            class="rounded-[11px] border border-rule bg-paper-raised p-6 shadow-paper [&>header]:mb-[26px] [&>header]:flex [&>header]:items-end [&>header]:justify-between [&>header]:gap-6"
          >
            <header>
              <div>
                <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">
                  Чтение · страница {readingBook.reading?.page ?? 1}
                </p>
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
            <div
              class="grid grid-cols-[.7fr_1.3fr] gap-7 pt-6 max-[640px]:grid-cols-1 [&_p]:text-ink-muted [&_form]:grid [&_form]:gap-[7px]"
            >
              <div>
                <h3>Место чтения</h3>
                <p>Bookshelf автоматически сохраняет страницу, масштаб и позицию.</p>
                <NumberField id="reading-page" label="Страница" min={1} bind:value={readingBook.reading.page} />
                <NumberField
                  id="reading-zoom"
                  label="Масштаб"
                  min={0.5}
                  max={4}
                  step={0.1}
                  bind:value={readingBook.reading.zoom}
                />
                <Button onclick={savePosition}>Сохранить сейчас</Button>
                <Button
                  onclick={() =>
                    run(
                      { kind: "completeReading", bookId: readingBook!.id },
                      "Чтение завершено; изучение остаётся активным",
                    )}>Завершить чтение</Button
                >
                <details>
                  <summary>Исправить оглавление</summary>{#each readingBook.outline ?? [] as item, index (item.id)}<div
                      class="my-2 grid grid-cols-[minmax(130px,1fr)_90px_minmax(130px,.8fr)_auto_auto] gap-1.5 max-[640px]:grid-cols-1"
                    >
                      <TextField id={`outline-title-${item.id}`} ariaLabel="Название раздела" bind:value={item.title} />
                      <NumberField
                        id={`outline-page-${item.id}`}
                        ariaLabel="Страница раздела"
                        min={1}
                        bind:value={item.page}
                      />
                      <SelectField
                        label="Родительский раздел"
                        value={item.parentId ?? ""}
                        placeholder="Без родителя"
                        options={readingBook.outline
                          .filter((candidate) => candidate.id !== item.id)
                          .map((parent) => ({ value: parent.id, label: parent.title }))}
                        onValueChange={(value) => (item.parentId = value || null)}
                      /><Button
                        disabled={index === 0}
                        onclick={() => {
                          const list = readingBook!.outline;
                          const previous = list[index - 1];
                          const current = list[index];
                          if (!previous || !current) return;
                          [list[index - 1], list[index]] = [current, previous];
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
                    <TextField
                      id="outline-title-new"
                      ariaLabel="Новый раздел"
                      bind:value={outlineTitle}
                      placeholder="Название главы"
                    />
                    <NumberField
                      id="outline-page-new"
                      ariaLabel="Страница нового раздела"
                      min={1}
                      bind:value={outlinePage}
                    />
                    <Button type="submit">Добавить</Button>
                  </form>
                  <Button onclick={saveOutline}>Сохранить правки</Button>
                </details>
              </div>
              <form onsubmit={captureDraft}>
                <h3>Черновая заметка</h3>
                <TextField id="section" label="Глава или раздел" bind:value={section} />
                <TextArea id="excerpt" label="Выделенный фрагмент" bind:value={excerpt} required />
                <TextArea id="context" label="Непосредственный контекст" bind:value={context} />
                <TextArea id="comment" label="Моя мысль (необязательно)" bind:value={comment} />
                <Button type="submit">В очередь разбора</Button>
              </form>
            </div>
          </section>
        {:else if library.books.length === 0}
          <section
            class="grid justify-items-center rounded-[14px] border border-rule bg-paper-raised px-7 py-14 text-center shadow-paper [&>div]:mb-5 [&>div]:grid [&>div]:h-16 [&>div]:w-[54px] [&>div]:place-items-center [&>div]:rounded-[4px_11px_11px_4px] [&>div]:bg-[#eaf1c8] [&>div]:text-[26px] [&>div]:shadow-[inset_5px_0_#b8d94a] [&>p:not(:first-child)]:max-w-[590px] [&>p:not(:first-child)]:leading-[1.65] [&>p:not(:first-child)]:text-ink-muted [&>small]:mt-[13px] [&>small]:text-ink-muted"
          >
            <div aria-hidden="true">▥</div>
            <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">
              Ваша первая закладка
            </p>
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
          <section
            class="grid gap-3.5 [&_article]:grid [&_article]:grid-cols-[104px_1fr] [&_article]:gap-6 [&_article]:border-b [&_article]:border-rule [&_article]:py-5 max-[640px]:[&_article]:grid-cols-[72px_1fr] [&_article>div:last-child]:self-center [&_article_p]:text-ink-muted"
            aria-label="Книги"
          >
            {#each library.books as book (book.id)}<article>
                <div
                  class="grid min-h-[138px] place-items-center rounded-[3px_9px_9px_3px] bg-[#283847] text-[#dce9a7] shadow-[inset_5px_0_#101923,0_8px_17px_#17212a24] max-[640px]:min-h-[104px] [&_span]:font-display [&_span]:text-[40px]"
                  aria-hidden="true"
                >
                  <span>{book.title.slice(0, 1)}</span>
                </div>
                <div>
                  <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">
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
                  <div class="mt-2.5 flex flex-wrap items-center gap-[9px]">
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
        {#if library.drafts.length === 0}<section
            class="grid justify-items-center rounded-[14px] border border-rule bg-paper-raised px-6 py-[42px] text-center shadow-paper [&>p]:max-w-[590px] [&>p]:leading-[1.65] [&>p]:text-ink-muted"
          >
            <h2>Очередь разобрана</h2>
            <p>Новые фрагменты можно сохранить, не выходя из просмотрщика.</p>
          </section>{:else}<section class="grid gap-3.5">
            {#each library.drafts as draft (draft.id)}<article
                class="rounded-[11px] border border-rule bg-paper-raised p-6 shadow-paper"
              >
                <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">
                  {bookTitle(draft.bookId)} · {draft.section} · стр. {draft.page}
                </p>
                <blockquote>{draft.excerpt}</blockquote>
                {#if draft.comment}<p>{draft.comment}</p>{/if}
                <TextArea id={`idea-${draft.id}`} label="Самостоятельная формулировка" bind:value={formulation} />
                <div class="mt-2.5 flex flex-wrap items-center gap-[9px]">
                  <Button variant="primary" onclick={() => resolveDraft(draft.id)}>Создать идею</Button>
                  <SelectField
                    label="Идея для присоединения"
                    bind:value={attachIdeaId}
                    placeholder="Выберите идею"
                    options={library.ideas.map((idea) => ({ value: idea.id, label: idea.formulation }))}
                  />
                  <Button
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
            <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">
              Идея · {bookTitle(openedIdea.bookId)} · {openedIdea.section}
            </p>
            <h2>{openedIdea.formulation}</h2>
            {#each openedIdea.fragments as fragment (`${fragment.page}-${fragment.excerpt}-${fragment.context}`)}<blockquote
              >
                стр. {fragment.page}: {fragment.excerpt}{#if fragment.context}<br /><small>{fragment.context}</small
                  >{/if}
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
            <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">Тема знаний</p>
            <h2>{openedTopic.name}</h2>
            {#each library.ideas.filter((idea) => idea.topicIds.includes(openedTopic!.id)) as idea (idea.id)}<article
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
            <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">
              Материал для передачи
            </p>
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
        <section
          class="mb-[26px] flex items-center justify-between gap-6 max-[640px]:flex-col max-[640px]:items-stretch [&_p]:text-ink-muted [&_form]:flex [&_form]:w-[min(460px,50%)] [&_form]:gap-2 max-[640px]:[&_form]:w-full"
        >
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
            <TextField
              id="topic-name"
              ariaLabel="Название темы"
              bind:value={topicName}
              placeholder="Например, архитектура данных"
            />
            <Button type="submit">Создать тему</Button>
          </form>
        </section>
        {#if library.ideas.length === 0}<section
            class="grid justify-items-center rounded-[14px] border border-rule bg-paper-raised px-6 py-[42px] text-center shadow-paper [&>p]:max-w-[590px] [&>p]:leading-[1.65] [&>p]:text-ink-muted"
          >
            <h2>Здесь появятся ваши идеи</h2>
            <p>Разберите черновую заметку, чтобы сохранить авторскую формулировку и источник.</p>
          </section>{:else}<IdeaWorkbench
            {library}
            {commands}
            {run}
            {bookTitle}
            onLibrary={(next) => {
              session.replaceFrom(next);
            }}
          />{#if library.materials.length}<section class="mt-5 grid gap-3.5" aria-label="Материалы для передачи">
              {#each library.materials as material (material.id)}<article
                  class="rounded-[11px] border border-rule bg-paper-raised p-6 shadow-paper"
                >
                  <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">
                    Материал для передачи
                  </p>
                  <h2>{material.title}</h2>
                  <div class="mt-2.5 flex flex-wrap items-center gap-[9px]">
                    <Button
                      onclick={async () => {
                        await navigator.clipboard.writeText(`# ${material.title}\n\n${material.idea}`);
                        session.feedback = "Материал скопирован";
                      }}>Скопировать</Button
                    ><Button onclick={() => exportMaterial(material.id, material.title)}>Сохранить Markdown</Button>
                  </div>
                </article>{/each}
            </section>{/if}{/if}
      {:else}
        <section class="mb-5 grid grid-cols-2 gap-4 max-[640px]:grid-cols-1">
          <article class="rounded-[11px] border border-rule bg-paper-raised p-6 shadow-paper">
            <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">Недельный ритм</p>
            <h2>{library.weeklySessionBudget || 3} сеанса</h2>
            <p>Без дедлайна книги, нормы страниц и штрафов.</p>
            <div class="flex gap-[7px]">
              {#each [2, 3, 4, 5] as count (count)}<button
                  class={library.weeklySessionBudget === count
                    ? "size-[42px] rounded-full border border-[#72843d] bg-[#eaf1c8] font-extrabold"
                    : "size-[42px] rounded-full border border-rule bg-white"}
                  onclick={() => run({ kind: "setStudyRhythm", weeklySessionBudget: count }, "Недельный ритм сохранён")}
                  >{count}</button
                >{/each}
            </div>
          </article>
          <article class="rounded-[11px] border border-rule bg-paper-raised p-6 shadow-paper">
            <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">Новый сеанс</p>
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
              <TextArea
                id="session-intention"
                ariaLabel="Намерение сеанса"
                bind:value={sessionIntention}
                placeholder="Разобрать две заметки и продолжить главу"
                required
              />
              <Button type="submit">Запланировать</Button>
            </form>
          </article>
        </section>
        {#if library.sessions.length}<section class="grid gap-3.5">
            {#each library.sessions as session (session.id)}<article
                class="flex items-center justify-between gap-5 border-b border-rule px-[5px] py-[15px] max-[640px]:flex-col max-[640px]:items-stretch [&_small]:mt-1 [&_small]:block [&_small]:text-ink-muted"
              >
                <div>
                  <b>{session.intention}</b><small>{sessionStatusLabel(session.status)}</small>
                </div>
                {#if session.status === "planned"}<div
                    class="flex items-center gap-[7px] max-[640px]:flex-col max-[640px]:items-stretch [&_input]:w-[210px] max-[640px]:[&_input]:w-full"
                  >
                    <Button
                      variant="primary"
                      onclick={() =>
                        run(
                          { kind: "startSession", sessionId: session.id },
                          "Сеанс начат; изменение долга будет измерено отсюда",
                        )}>Начать сеанс</Button
                    >
                    <SelectField
                      label="Решение по пропущенному сеансу"
                      bind:value={rescheduledSessionStatus}
                      options={[
                        { value: "moved", label: "Перенести" },
                        { value: "replaced", label: "Заменить" },
                        { value: "cancelled", label: "Отменить" },
                      ]}
                    />
                    <TextField
                      id={`session-reason-${session.id}`}
                      ariaLabel="Причина решения"
                      bind:value={rescheduledSessionReason}
                      placeholder="Почему план изменился"
                    />
                    <Button
                      onclick={() =>
                        run(
                          {
                            kind: "resolveSession",
                            sessionId: session.id,
                            status: rescheduledSessionStatus,
                            reason: rescheduledSessionReason,
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
            class="mt-5 grid gap-[7px] rounded-[11px] border border-rule bg-paper-raised p-6 shadow-paper"
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
            <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">
              Явное завершение изучения
            </p>
            <h2>Ретроспектива «{activeBook.title}»</h2>
            <TextArea
              id="retrospective"
              label="Результаты применения и изменения в понимании или действиях"
              bind:value={retrospective}
            />
            <fieldset>
              <legend>3–7 значимых идей</legend>
              <div class="grid gap-2 py-2">
                {#each library.ideas.filter((idea) => idea.bookId === activeBook?.id) as idea (idea.id)}
                  <CheckboxField
                    label={idea.formulation}
                    checked={significantIdeaIds.includes(idea.id)}
                    onCheckedChange={(checked) => setSignificantIdea(idea.id, checked)}
                  />
                {/each}
              </div>
            </fieldset>
            <TextArea
              id="continuing"
              label="Продолжающиеся эксперименты или восстановления"
              bind:value={continuingWork}
            />
            <TextArea id="debt-decision" label="Решение по оставшемуся долгу" bind:value={debtDecision} />
            <Button variant="primary" type="submit">Завершить изучение</Button>
          </form>{/if}
      {/if}
    {/if}
  </div>
</WorkspaceShell>
