<script lang="ts">
  import { onMount } from "svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { Button, StatePanel, StatusMessage, TextField } from "@/shared/ui";
  import { IdeaWorkbench } from "@/features/idea-workbench";
  import {
    bookFilePath,
    commandErrorMessage,
    executeLibraryAction,
    exportLibraryArchive,
    importLibraryArchive,
    installSignedUpdate,
    importPdf,
    loadLibrary,
    saveWorkspaceNote,
    restoreLatestSnapshot,
    searchLibrary,
    type Book,
    type Idea,
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

  let debt = $derived((library?.drafts.length ?? 0) + (library?.reviews.filter((item) => item.pending).length ?? 0));
  let activeBook = $derived(library?.books.find((book) => book.id === library?.activeStudyBookId));
  let nextStep = $derived(
    debt > 0 ? "Разобрать ближайшую заметку" : activeBook ? `Продолжить «${activeBook.title}»` : library?.books.length ? "Выбрать книгу для изучения" : "Импортировать первую книгу",
  );

  onMount(async () => {
    try {
      library = await loadLibrary();
      note = library.workspaceNote;
    } catch (cause) {
      error = commandErrorMessage(cause);
    } finally {
      loading = false;
    }
  });

  async function run(action: LibraryAction, success = "Изменения сохранены") {
    busy = true; error = ""; feedback = "";
    try { library = await executeLibraryAction(action); feedback = success; }
    catch (cause) { error = commandErrorMessage(cause); }
    finally { busy = false; }
  }

  async function importBook() {
    error = "";
    const path = await open({ multiple: false, filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (!path) return;
    busy = true;
    try { library = await importPdf(path); feedback = "Книга скопирована в личную библиотеку"; view = "library"; }
    catch (cause) { error = commandErrorMessage(cause); }
    finally { busy = false; }
  }

  async function openBook(book: Book) {
    busy = true; error = "";
    try { bookUrl = convertFileSrc(await bookFilePath(book.id)); readingBook = book; }
    catch (cause) { error = commandErrorMessage(cause); }
    finally { busy = false; }
  }

  async function savePosition() {
    if (!readingBook) return;
    await run({ kind: "updateReading", bookId: readingBook.id, page: readingBook.reading?.page || 1, zoom: readingBook.reading?.zoom || 1, scroll: 0 }, "Место чтения сохранено");
  }

  async function captureDraft(event: SubmitEvent) {
    event.preventDefault(); if (!readingBook) return;
    await run({ kind: "captureDraft", bookId: readingBook.id, section, page: readingBook.reading?.page || 1, excerpt, context, comment }, "Черновая заметка добавлена в очередь");
    if (!error) { excerpt = ""; context = ""; comment = ""; }
  }

  async function resolveDraft(draftId: string) {
    await run({ kind: "resolveDraftAsIdea", draftId, formulation, section, assignments: ["recall"] }, "Черновик стал самостоятельной идеей");
    if (!error) formulation = "";
  }

  async function saveNote(event: SubmitEvent) {
    event.preventDefault(); busy = true; saved = false; error = "";
    try { library = await saveWorkspaceNote(note); saved = true; }
    catch (cause) { error = commandErrorMessage(cause); }
    finally { busy = false; }
  }

  async function search() {
    try { results = await searchLibrary(searchQuery); }
    catch (cause) { error = commandErrorMessage(cause); }
  }

  async function exportArchive() {
    const path = await save({ defaultPath: "bookshelf-library.age", filters: [{ name: "Bookshelf archive", extensions: ["age"] }] });
    if (!path) return;
    busy = true; error = "";
    try { await exportLibraryArchive(path, archivePassword); feedback = "Зашифрованный архив сохранён"; }
    catch (cause) { error = commandErrorMessage(cause); }
    finally { busy = false; }
  }

  async function importArchive() {
    const path = await open({ multiple: false, filters: [{ name: "Bookshelf archive", extensions: ["age"] }] });
    if (!path) return;
    busy = true; error = "";
    try { library = await importLibraryArchive(path, archivePassword); feedback = "Личная библиотека восстановлена; вход в Codex потребуется выполнить заново"; }
    catch (cause) { error = commandErrorMessage(cause); }
    finally { busy = false; }
  }

  function navigate(target: View) { view = target; readingBook = null; settingsOpen = false; feedback = ""; error = ""; }
  function bookTitle(bookId: string) { return library?.books.find((book) => book.id === bookId)?.title ?? "Книга"; }
  function reviewPackage(idea: Idea) {
    const fragment = idea.fragments[0];
    return [`Проверь идею и укажи возможные пробелы.`, `Источник: ${bookTitle(idea.bookId)}, ${idea.section}${fragment ? `, стр. ${fragment.page}` : ""}`, fragment ? `Фрагмент: ${fragment.excerpt}` : "", `Моя формулировка: ${idea.formulation}`, `Вопрос: при каких условиях идея не сработает?`, `Критерии: точность, ограничения, связь с источником.`].filter(Boolean).join("\n\n");
  }
</script>

<svelte:head><title>Bookshelf — личная библиотека</title><meta name="description" content="Локальное пространство для системного изучения технических книг" /></svelte:head>

<div class="app-shell">
  <aside class="spine">
    <button class="brand" aria-label="Открыть библиотеку" onclick={() => navigate("library")}><span aria-hidden="true">B</span><b>Bookshelf</b></button>
    <nav aria-label="Основная навигация">
      <button class:active={view === "library"} aria-current={view === "library" ? "page" : undefined} onclick={() => navigate("library")}><span>Библиотека</span><small>{library?.books.length ?? 0}</small></button>
      <button class:active={view === "queue"} aria-current={view === "queue" ? "page" : undefined} onclick={() => navigate("queue")}><span>Очередь разбора</span><small>{library?.drafts.length ?? 0}</small></button>
      <button class:active={view === "ideas"} aria-current={view === "ideas" ? "page" : undefined} onclick={() => navigate("ideas")}><span>Идеи</span><small>{library?.ideas.length ?? 0}</small></button>
      <button class:active={view === "study"} aria-current={view === "study" ? "page" : undefined} onclick={() => navigate("study")}><span>Изучение</span></button>
    </nav>
    <button class="settings" aria-expanded={settingsOpen} onclick={() => settingsOpen = !settingsOpen}>Настройки</button>
    <p class="local"><i></i> Данные остаются на этом компьютере</p>
  </aside>

  <main>
    {#if loading}
      <StatePanel live>Открываем личную библиотеку…</StatePanel>
    {:else if error && !library}
      <StatePanel tone="danger"><strong>Библиотека не открылась</strong><p>{error}</p></StatePanel>
    {:else if library}
      <header class="topbar">
        <div><p class="eyebrow">Личная библиотека</p><h1>{view === "library" ? "Рабочий стол читателя" : view === "queue" ? "Очередь разбора" : view === "ideas" ? "Идеи книги" : "Ритм изучения"}</h1></div>
        <Button variant="primary" onclick={importBook} disabled={busy}>Импортировать PDF</Button>
      </header>

      {#if error}<StatusMessage tone="danger">{error}</StatusMessage>{/if}
      {#if feedback}<div class="feedback" role="status">{feedback}</div>{/if}

      {#if settingsOpen}
        <section class="settings-panel" aria-label="Настройки">
          <div><p class="eyebrow">Настройки</p><h2>Личное напоминание</h2><p>Короткая запись для себя — например, с чего начать в следующий раз.</p></div>
          <form onsubmit={saveNote}>
            <TextField id="workspace-note" label="Личное напоминание" bind:value={note} placeholder="Например, вернуться к главе 2" maxlength={240} disabled={busy} />
            <div class="form-actions"><Button type="submit" disabled={busy}>Сохранить</Button>{#if saved}<StatusMessage tone="success">Сохранено локально</StatusMessage>{/if}</div>
          </form>
          <div><p class="eyebrow">Перенос и восстановление</p><h2>Зашифрованный архив</h2><p>Архив содержит рабочее состояние и PDF. Забытый пароль восстановить невозможно. Данные входа Codex не переносятся.</p></div>
          <div><TextField id="archive-password" label="Пароль архива" bind:value={archivePassword} placeholder="Не менее 8 символов" disabled={busy} type="password" /><div class="form-actions"><Button onclick={exportArchive} disabled={busy || archivePassword.length < 8}>Экспортировать</Button><Button onclick={importArchive} disabled={busy || archivePassword.length < 8}>Импортировать</Button><Button onclick={async () => { try { library = await restoreLatestSnapshot(); feedback = "Последний снимок восстановлен"; } catch (cause) { error = commandErrorMessage(cause); } }}>Восстановить снимок</Button></div></div>
          <div><p class="eyebrow">Обновления</p><h2>Совместимая версия Bookshelf</h2><p>Устанавливаются только обновления с проверяемой подписью. При ошибке текущая версия и личная библиотека остаются без изменений.</p></div>
          <div class="form-actions"><Button onclick={async () => { try { const installed = await installSignedUpdate(); feedback = installed ? "Подписанное обновление установлено" : "У вас актуальная версия"; } catch (cause) { error = commandErrorMessage(cause); } }}>Проверить обновления</Button></div>
        </section>
      {/if}

      {#if view === "library"}
        <section class="bookmark" aria-label="Следующий шаг">
          <div><span>Следующий шаг</span><strong>{nextStep}</strong><p>Рекомендация учитывает доступную работу, но выбор всегда остаётся за вами.</p></div>
          {#if activeBook}<Button onclick={() => openBook(activeBook)}>Продолжить чтение</Button>{:else if library.books.length === 0}<Button onclick={importBook}>Выбрать PDF</Button>{/if}
          <div class="debt"><small>Долг изучения</small><b>{debt}</b></div>
        </section>

        <form class="search" onsubmit={(event) => { event.preventDefault(); search(); }}>
          <label for="search">Поиск по книгам, идеям и источникам</label><div><input id="search" bind:value={searchQuery} placeholder="Название или формулировка" /><Button type="submit">Найти</Button></div>
        </form>
        {#if searchQuery && results.length === 0}<p class="muted">Совпадений пока нет.</p>{/if}
        {#if results.length}<ul class="results">{#each results as result}<li><small>{result.kind === "book" ? "Книга" : "Идея"}</small><b>{result.title}</b><span>{result.context}</span></li>{/each}</ul>{/if}

        {#if readingBook}
          <section class="reader">
            <header><div><p class="eyebrow">Чтение · страница {readingBook.reading?.page ?? 1}</p><h2>{readingBook.title}</h2></div><Button onclick={() => readingBook = null}>Закрыть</Button></header>
            <iframe title={`PDF: ${readingBook.title}`} src={bookUrl}></iframe>
            <div class="reader-tools">
              <div><h3>Место чтения</h3><p>Bookshelf сохранит страницу и масштаб между запусками.</p><Button onclick={savePosition}>Сохранить место</Button> <Button onclick={() => run({ kind: "completeReading", bookId: readingBook!.id }, "Чтение завершено; изучение остаётся активным")}>Завершить чтение</Button></div>
              <form onsubmit={captureDraft}><h3>Черновая заметка</h3><label for="section">Глава или раздел</label><input id="section" bind:value={section} /><label for="excerpt">Выделенный фрагмент</label><textarea id="excerpt" bind:value={excerpt} required></textarea><label for="context">Непосредственный контекст</label><textarea id="context" bind:value={context}></textarea><label for="comment">Моя мысль (необязательно)</label><textarea id="comment" bind:value={comment}></textarea><Button type="submit">В очередь разбора</Button></form>
            </div>
          </section>
        {:else if library.books.length === 0}
          <section class="empty"><div aria-hidden="true">▥</div><p class="eyebrow">Ваша первая закладка</p><h2>Начните с одной важной книги</h2><p>Импортируйте PDF с текстовым слоем. Bookshelf сделает локальную копию, сохранит место чтения и поможет превратить фрагменты в применимые идеи.</p><Button variant="primary" onclick={importBook}>Импортировать первую книгу</Button><small>PDF не покинет этот компьютер</small></section>
        {:else}
          <section class="books" aria-label="Книги">{#each library.books as book}<article><div class="cover" aria-hidden="true"><span>{book.title.slice(0, 1)}</span></div><div><p class="eyebrow">{book.readingCompleted ? "Чтение завершено" : `Страница ${book.reading?.page ?? 1}`}</p><h2>{book.title}</h2><p>{book.id === library.activeStudyBookId ? "Активное изучение" : book.studyCompleted ? "Изучение завершено" : "Готова к изучению"}</p><div class="card-actions"><Button onclick={() => openBook(book)}>{book.reading?.page > 1 ? "Продолжить" : "Открыть"}</Button>{#if book.id !== library.activeStudyBookId && !book.studyCompleted}<Button onclick={() => run({ kind: "activateStudy", bookId: book.id }, "Книга выбрана для активного изучения")}>Изучать</Button>{/if}</div></div></article>{/each}</section>
        {/if}
      {:else if view === "queue"}
        {#if library.drafts.length === 0}<section class="empty compact"><h2>Очередь разобрана</h2><p>Новые фрагменты можно сохранить, не выходя из просмотрщика.</p></section>{:else}<section class="stack">{#each library.drafts as draft}<article class="work-card"><p class="eyebrow">{bookTitle(draft.bookId)} · {draft.section} · стр. {draft.page}</p><blockquote>{draft.excerpt}</blockquote>{#if draft.comment}<p>{draft.comment}</p>{/if}<label for={`idea-${draft.id}`}>Самостоятельная формулировка</label><textarea id={`idea-${draft.id}`} bind:value={formulation}></textarea><div class="card-actions"><Button variant="primary" onclick={() => resolveDraft(draft.id)}>Создать идею</Button><Button onclick={() => run({ kind: "discardDraft", draftId: draft.id }, "Черновая заметка удалена")}>Удалить</Button></div></article>{/each}</section>{/if}
      {:else if view === "ideas"}
        <section class="section-head"><div><h2>Темы знаний</h2><p>Темы и связи появляются только после вашего подтверждения.</p></div><form onsubmit={(event) => { event.preventDefault(); run({ kind: "createTopic", name: topicName }, "Тема создана"); topicName = ""; }}><input aria-label="Название темы" bind:value={topicName} placeholder="Например, архитектура данных" /><Button type="submit">Создать тему</Button></form></section>
        {#if library.ideas.length === 0}<section class="empty compact"><h2>Здесь появятся ваши идеи</h2><p>Разберите черновую заметку, чтобы сохранить авторскую формулировку и источник.</p></section>{:else}<IdeaWorkbench {library} {run} {bookTitle} /><section class="stack review-packages">{#each library.ideas as idea}<details class="work-card"><summary>Пакет ручной проверки: {idea.formulation}</summary><pre>{reviewPackage(idea)}</pre><Button onclick={async () => { await navigator.clipboard.writeText(reviewPackage(idea)); feedback = "Подтверждённый пакет скопирован"; }}>Скопировать подтверждённый пакет</Button></details>{/each}</section>{/if}
      {:else}
        <section class="study-grid"><article class="work-card"><p class="eyebrow">Недельный ритм</p><h2>{library.weeklySessionBudget || 3} сеанса</h2><p>Без дедлайна книги, нормы страниц и штрафов.</p><div class="rhythm">{#each [2, 3, 4, 5] as count}<button class:chosen={library.weeklySessionBudget === count} onclick={() => run({ kind: "setStudyRhythm", weeklySessionBudget: count }, "Недельный ритм сохранён")}>{count}</button>{/each}</div></article><article class="work-card"><p class="eyebrow">Новый сеанс</p><h2>С каким намерением?</h2><form onsubmit={(event) => { event.preventDefault(); run({ kind: "planSession", intention: sessionIntention, plannedAt: Math.floor(Date.now() / 1000) }, "Сеанс запланирован"); sessionIntention = ""; }}><textarea aria-label="Намерение сеанса" bind:value={sessionIntention} placeholder="Разобрать две заметки и продолжить главу" required></textarea><Button type="submit">Запланировать</Button></form></article></section>
        {#if library.sessions.length}<section class="stack">{#each library.sessions as session}<article class="session"><div><b>{session.intention}</b><small>{session.status === "planned" ? "Запланирован" : session.status}</small></div>{#if session.status === "planned"}<Button onclick={() => run({ kind: "resolveSession", sessionId: session.id, status: "completed", reason: "" }, `Сеанс завершён. Долг ${library!.lastDebtChange > 0 ? "вырос" : library!.lastDebtChange < 0 ? "уменьшился" : "не изменился"}`)}>Проведён</Button>{/if}</article>{/each}</section>{/if}
      {/if}
    {/if}
  </main>
</div>
