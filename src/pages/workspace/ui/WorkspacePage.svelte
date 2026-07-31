<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { Button, DialogModal, TextArea } from "@/shared/ui";
  import {
    commandErrorMessage,
    type Book,
    type CompletionWorkDecision,
    type ExperimentStatus,
    type Idea,
    type IdeaAssignment,
    type IdeaRelation,
    type LibraryAction,
    type LibraryState,
    type ReviewDecision,
    type SourceFragment,
  } from "@/shared/api";
  import { createWorkspaceCommands, type WorkspaceCommands } from "../api/workspace-commands";
  import type { WorkspaceContext } from "../model/workspace-context";
  import CompletionView from "./CompletionView.svelte";
  import DashboardView from "./DashboardView.svelte";
  import BookView from "./BookView.svelte";
  import DraftsView from "./DraftsView.svelte";
  import KnowledgeView from "./KnowledgeView.svelte";
  import LibraryView from "./LibraryView.svelte";
  import PracticeView from "./PracticeView.svelte";
  import ReaderView from "./ReaderView.svelte";
  import SettingsView from "./SettingsView.svelte";

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

  let libraryFilter = $state("all");
  let librarySort = $state("recent");
  let selectedDraftId = $state("");
  let selectedTopic = $state("all");
  let completionStep = $state(1);
  let retrospective = $state("");
  let significantIdeas = $state<string[]>([]);
  let unfinishedWorkDecision = $state("");
  let continuingWork = $state("");
  let completionWorkDecisions = $state<CompletionWorkDecision[]>([]);
  let codexReviewOpen = $state(false);
  let codexPackage = $state("");
  let codexReviewResult = $state("");
  let ideaFormulation = $state("");
  let reviewConclusion = $state("");
  let deleteBookOpen = $state(false);
  let settingsSection = $state<"interface" | "library" | "backups" | "ai">("interface");
  let updateStatus = $state("");
  let diagnosticStatus = $state("");
  let diagnosticEntries = $state<string[]>([]);

  let readerSidebar = $state<"note" | "outline" | "search" | null>(null);
  let readerSidebarWidth = $state(400);
  let readerZoom = $state(1.15);
  let readerPage = $state(286);
  let readerMode = $state<"muted" | "original" | "dark">("muted");
  let readerImages = $state(true);
  let readerSearch = $state("");
  let readerSearchResults = $state<{ page: number; excerpt: string }[]>([]);
  let saveState = $state<"saved" | "saving" | "error">("saved");
  let sidebarTrigger = $state<HTMLButtonElement | null>(null);
  let readerDocumentUrl = $state<string | null>(null);

  const activeBook = $derived(library?.books.find((book) => book.id === library?.activeStudyBookId) ?? null);
  const selectedBook = $derived(
    library?.books.find((book) => book.id === resourceId) ?? activeBook ?? library?.books[0] ?? null,
  );
  const selectedIdea = $derived(library?.ideas.find((idea) => idea.id === resourceId) ?? library?.ideas[0] ?? null);
  const focusedDraft = $derived(
    library?.drafts.find((draft) => draft.id === selectedDraftId) ?? library?.drafts[0] ?? null,
  );
  const pendingIdeaReview = $derived(
    library?.reviews.find(
      (review) => review.ideaId === selectedIdea?.id && review.requestKind === "ideaReview" && review.pending,
    ) ?? null,
  );
  const completionWorkItems = $derived.by(() => {
    if (!library || !selectedBook) return [];
    const ideaIds = new Set(library.ideas.filter((idea) => idea.bookId === selectedBook.id).map((idea) => idea.id));
    return [
      ...library.drafts
        .filter((draft) => draft.bookId === selectedBook.id)
        .map((draft) => ({ id: draft.id, kind: "draft" as const, label: `Черновик · ${draft.section}` })),
      ...library.reviews
        .filter((review) => review.pending && ideaIds.has(review.ideaId))
        .map((review) => ({ id: review.id, kind: "review" as const, label: "Ожидающая проверка идеи" })),
      ...library.recalls
        .filter((recall) => ideaIds.has(recall.ideaId))
        .map((recall) => ({ id: recall.id, kind: "recall" as const, label: "Следующее восстановление знания" })),
      ...library.experiments
        .filter(
          (experiment) => ideaIds.has(experiment.ideaId) && !["completed", "cancelled"].includes(experiment.status),
        )
        .map((experiment) => ({
          id: experiment.id,
          kind: "experiment" as const,
          label: `Эксперимент · ${experiment.situation}`,
        })),
    ];
  });
  const unfinishedCount = $derived(
    (library?.drafts.length ?? 0) +
      (library?.experiments.filter((experiment) => !["completed", "cancelled"].includes(experiment.status)).length ??
        0) +
      (library?.reviews.filter((review) => review.pending).length ?? 0),
  );
  const filteredBooks = $derived.by(() => {
    if (!library) return [];
    const snapshot = library;
    const books = snapshot.books.filter((book) => {
      if (libraryFilter === "all") return !book.archived;
      if (libraryFilter === "archived") return book.archived;
      if (book.archived) return false;
      if (libraryFilter === "active") return book.id === snapshot.activeStudyBookId;
      if (libraryFilter === "completed") return book.studyStatus === "completed";
      if (libraryFilter === "ready") return book.readingCompleted && book.studyStatus !== "completed";
      if (libraryFilter === "paused") return book.studyStatus === "paused";
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
      const query = new URLSearchParams(location.search);
      selectedDraftId = query.get("draft") ?? "";
      selectedTopic = query.get("topic") ?? "all";
      if (selectedBook) {
        readerPage = Number(query.get("sourcePage")) || selectedBook.reading.page;
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
        const completionDraft = library.completionDrafts.find((draft) => draft.bookId === selectedBook.id);
        if (completionDraft) {
          completionStep = completionDraft.step;
          significantIdeas = completionDraft.significantIdeaIds;
          retrospective = completionDraft.retrospective;
          unfinishedWorkDecision = completionDraft.unfinishedWorkDecision;
          continuingWork = completionDraft.continuingWork;
          completionWorkDecisions = [...completionDraft.workDecisions];
        }
      }
      if (selectedIdea) {
        ideaFormulation = selectedIdea.formulation;
      }
    } catch (cause) {
      error = commandErrorMessage(cause);
      recordDiagnostic("library-load", error);
    } finally {
      loading = false;
    }
  });

  async function executeLibraryAction(action: LibraryAction, message = "Сохранено"): Promise<boolean> {
    if (!commands) return false;
    busy = true;
    error = "";
    feedback = "";
    try {
      library = await commands.execute(action);
      feedback = message;
      return true;
    } catch (cause) {
      error = commandErrorMessage(cause);
      recordDiagnostic(`action-${action.kind}`, error);
      return false;
    } finally {
      busy = false;
    }
  }

  async function importBook() {
    if (!commands) return;
    busy = true;
    try {
      const result = await commands.importPdf();
      if (result) {
        library = result.state;
        feedback = result.duplicate ? "Эта редакция PDF уже есть в библиотеке" : "PDF импортирован";
        await goto(resolve("/library/[bookId]", { bookId: result.bookId }));
      }
    } catch (cause) {
      error = commandErrorMessage(cause);
    } finally {
      busy = false;
    }
  }

  function handleGlobalKeydown(event: KeyboardEvent) {
    if (context === "reader" && event.key === "Escape" && readerSidebar) {
      closeReaderSidebar();
    }
    if (context === "reader" && event.ctrlKey && event.key.toLowerCase() === "f") {
      event.preventDefault();
      setReaderSidebar("search");
    }
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
    await executeLibraryAction(
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

  async function saveReaderDraft(
    excerpt: string,
    comment: string,
    fragments: SourceFragment[],
  ): Promise<string | null> {
    if (!selectedBook || selectedBook.studyStatus === "completed" || !excerpt.trim()) return null;
    const existingDraftIds = new Set(library?.drafts.map((draft) => draft.id) ?? []);
    const saved = await executeLibraryAction(
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
            page: readerPage,
            excerpt,
            context: "Фрагмент сохранён из непрерывного режима чтения.",
            comment,
          },
      "Черновая заметка сохранена",
    );
    if (!saved) return null;
    const createdDraft = library?.drafts.find((draft) => !existingDraftIds.has(draft.id)) ?? null;
    return createdDraft?.id ?? null;
  }

  async function createReaderIdea(draftId: string, formulation: string): Promise<boolean> {
    if (!draftId || !formulation.trim()) return false;
    const draft = library?.drafts.find((item) => item.id === draftId);
    if (!draft) return false;
    const created = await executeLibraryAction(
      {
        kind: "resolveDraftAsIdea",
        draftId: draft.id,
        formulation,
        section: draft.section,
        assignments: [],
      },
      "Идея сохранена; назначение идеи можно выбрать позже",
    );
    return created;
  }

  async function saveReaderPosition(page = readerPage) {
    if (!selectedBook) return;
    readerPage = page;
    saveState = "saving";
    saveState = (await executeLibraryAction(
      { kind: "updateReading", bookId: selectedBook.id, page, zoom: readerZoom, scroll: 0.32 },
      "",
    ))
      ? "saved"
      : "error";
  }

  function changeReaderZoom(delta: number) {
    readerZoom = Math.min(2, Math.max(0.5, readerZoom + delta));
    void saveReaderPosition();
  }

  function openSavedSource(draftId: string, source: SourceFragment) {
    void saveReaderPosition(source.page).then(() => goto(resolve(`/drafts?draft=${encodeURIComponent(draftId)}`)));
  }

  function savePdfPosition(page: number, scroll: number) {
    readerPage = page;
    saveState = "saving";
    void executeLibraryAction(
      { kind: "updateReading", bookId: selectedBook!.id, page, zoom: readerZoom, scroll },
      "",
    ).then((saved) => (saveState = saved ? "saved" : "error"));
  }

  async function resolveDraft(formulation: string) {
    if (!focusedDraft || !formulation.trim()) return;
    await executeLibraryAction(
      {
        kind: "resolveDraftAsIdea",
        draftId: focusedDraft.id,
        formulation,
        section: focusedDraft.section,
        assignments: [],
      },
      "Идея сформулирована; источник сохранён",
    );
  }

  async function attachFocusedDraft(ideaId: string) {
    const idea = library?.ideas.find((item) => item.id === ideaId && item.bookId === focusedDraft?.bookId);
    if (!focusedDraft || !idea) return;
    await executeLibraryAction(
      { kind: "attachDraftToIdea", draftId: focusedDraft.id, ideaId: idea.id },
      "Источник присоединён к идее",
    );
  }

  async function exportFocusedDraft() {
    if (!focusedDraft || !commands) return;
    try {
      const snapshot = await commands.exportDraft(focusedDraft.id);
      if (snapshot) {
        library = snapshot;
        feedback = "Черновая заметка экспортирована";
      }
    } catch (cause) {
      error = commandErrorMessage(cause);
    }
  }

  async function prepareIdeaReview() {
    if (!selectedIdea || !commands) return;
    try {
      codexPackage = await commands.prepareReview(selectedIdea.id, "ideaReview");
      codexReviewResult = "";
      codexReviewOpen = true;
    } catch (cause) {
      error = commandErrorMessage(cause);
    }
  }

  async function runIdeaReview() {
    if (!selectedIdea || !commands || !codexPackage) return;
    try {
      library = await commands.runReview(selectedIdea.id, "ideaReview", codexPackage);
      codexReviewResult = "Проверка завершена. Ответ сохранён как ожидающее вашего решения замечание.";
    } catch (cause) {
      codexReviewResult = commandErrorMessage(cause);
    }
  }

  async function saveIdea(formulation: string, assignments: IdeaAssignment[]) {
    if (!selectedIdea || !formulation.trim() || assignments.length === 0) return;
    await executeLibraryAction(
      {
        kind: "updateIdea",
        ideaId: selectedIdea.id,
        formulation,
        assignments: [...assignments],
      },
      "Идея и назначения сохранены",
    );
  }

  async function linkSelectedIdea(relatedIdeaId: string, relation: IdeaRelation) {
    if (!selectedIdea || !relatedIdeaId) return;
    await executeLibraryAction(
      { kind: "linkIdeas", fromIdeaId: selectedIdea.id, toIdeaId: relatedIdeaId, relation },
      "Связь идей подтверждена",
    );
  }

  async function resolveIdeaReview(decision: Exclude<ReviewDecision, "pending">) {
    if (!selectedIdea) return;
    const saved = await executeLibraryAction(
      {
        kind: "resolveReview",
        ideaId: selectedIdea.id,
        requestKind: "ideaReview",
        decision,
        formulation: ideaFormulation,
        conclusion: reviewConclusion,
      },
      decision === "refined"
        ? "Уточнённая формулировка сохранена"
        : decision === "unchanged"
          ? "Идея оставлена без изменений"
          : "Проверка сохранена на потом",
    );
    if (saved && decision !== "later") codexReviewOpen = false;
  }

  async function permanentlyDeleteSelectedBook() {
    if (!selectedBook || !commands) return;
    busy = true;
    try {
      library = await commands.deleteBook(selectedBook.id);
      deleteBookOpen = false;
      await goto(resolve("/library"));
    } catch (cause) {
      error = commandErrorMessage(cause);
    } finally {
      busy = false;
    }
  }

  async function completeRecall(answer: string, rating: "confident" | "partial" | "notRecalled") {
    const recall = library?.recalls.toSorted((a, b) => a.nextAt - b.nextAt)[0];
    const idea = library?.ideas.find((item) => item.id === recall?.ideaId);
    if (!idea || !answer.trim()) return;
    await executeLibraryAction(
      { kind: "completeRecall", ideaId: idea.id, answer, rating },
      "Решение восстановления сохранено",
    );
  }

  async function rescheduleRecall(days: number) {
    const recall = library?.recalls[0];
    if (!recall) return;
    await executeLibraryAction(
      { kind: "rescheduleRecall", recallId: recall.id, nextAt: recall.nextAt + days * 86_400 },
      `Следующее восстановление перенесено на ${days} дней`,
    );
  }

  async function startRecallNow() {
    const recall = library?.recalls[0];
    if (!recall) return;
    await executeLibraryAction(
      { kind: "rescheduleRecall", recallId: recall.id, nextAt: Math.floor(Date.now() / 1_000) },
      "Восстановление готово сейчас",
    );
  }

  async function createExperiment(draft: { ideaId: string; situation: string; action: string; nextStep: string }) {
    if (!draft.ideaId || !draft.situation.trim() || !draft.action.trim()) return;
    await executeLibraryAction(
      {
        kind: "createExperiment",
        ideaId: draft.ideaId,
        situation: draft.situation,
        action: draft.action,
        nextStep: draft.nextStep,
      },
      "Замысел эксперимента сохранён",
    );
  }

  async function advanceExperiment(
    experimentId: string,
    status: ExperimentStatus,
    draft: {
      situation: string;
      action: string;
      result: string;
      conclusion: string;
      cancellationReason: string;
      nextStep: string;
    },
  ) {
    const experiment = library?.experiments.find((item) => item.id === experimentId);
    if (!experiment) return;
    await executeLibraryAction(
      {
        kind: "advanceExperiment",
        experimentId: experiment.id,
        status,
        situation: draft.situation,
        action: draft.action,
        result: draft.result,
        conclusion: draft.conclusion,
        cancellationReason: draft.cancellationReason,
        nextStep: draft.nextStep || experiment.nextStep,
      },
      "Состояние эксперимента сохранено",
    );
  }

  async function saveCompletionStep(nextStep: number) {
    if (!selectedBook) return;
    await executeLibraryAction(
      {
        kind: "saveStudyCompletionDraft",
        draft: {
          bookId: selectedBook.id,
          step: nextStep,
          readingConfirmed: nextStep > 1,
          significantIdeaIds: [...significantIdeas],
          retrospective,
          unfinishedWorkDecision,
          continuingWork,
          workDecisions: completionWorkDecisions.map((decision) => ({ ...decision })),
        },
      },
      "Черновик итога сохранён",
    );
    completionStep = nextStep;
  }

  async function finishStudy() {
    if (!selectedBook) return;
    await executeLibraryAction(
      {
        kind: "completeStudy",
        bookId: selectedBook.id,
        retrospective,
        significantIdeaIds: [...significantIdeas],
        continuingWork,
        unfinishedWorkDecision,
        workDecisions: completionWorkDecisions.map((decision) => ({ ...decision })),
      },
      "Изучение завершено; продолжающаяся работа сохранена",
    );
  }

  async function restoreBackup(): Promise<string> {
    if (!commands) return "Команды библиотеки недоступны";
    try {
      library = await commands.restoreBackup();
      return "Последний snapshot восстановлен";
    } catch (cause) {
      return commandErrorMessage(cause);
    }
  }

  async function exportArchive(password: string): Promise<string> {
    if (!commands) return "Команды библиотеки недоступны";
    try {
      return (await commands.exportArchive(password)) ? "Переносимый архив сохранён" : "Экспорт отменён";
    } catch (cause) {
      return commandErrorMessage(cause);
    }
  }

  async function importArchive(password: string): Promise<string> {
    if (!commands) return "Команды библиотеки недоступны";
    try {
      const snapshot = await commands.importArchive(password);
      if (snapshot) library = snapshot;
      return snapshot ? "Архив импортирован" : "Импорт отменён";
    } catch (cause) {
      return commandErrorMessage(cause);
    }
  }

  function recordDiagnostic(scope: string, message: string) {
    diagnosticEntries = [...diagnosticEntries, `${new Date().toISOString()} ${scope}: ${message}`].slice(-100);
  }

  async function checkForUpdate() {
    if (!commands) return;
    updateStatus = "Проверяем подписанное обновление…";
    try {
      updateStatus = (await commands.checkForUpdate())
        ? "Обновление установлено; перезапустите Bookshelf"
        : "Установлена актуальная версия";
    } catch (cause) {
      updateStatus = commandErrorMessage(cause);
      recordDiagnostic("signed-update", updateStatus);
    }
  }

  async function exportDiagnostics() {
    if (!commands) return;
    diagnosticStatus = "Экспортируем локальный журнал…";
    try {
      diagnosticStatus = (await commands.exportDiagnostics(diagnosticEntries))
        ? "Диагностический журнал экспортирован"
        : "Экспорт журнала отменён";
    } catch (cause) {
      diagnosticStatus = commandErrorMessage(cause);
    }
  }

  function bookStatus(book: Book): string {
    if (book.archived) return "В архиве";
    if (book.studyStatus === "completed") return "Завершено";
    if (book.id === library?.activeStudyBookId) return "Активное изучение";
    if (book.readingCompleted) return "Готово к завершению";
    return "Приостановлено";
  }

  function bookForIdea(idea: Idea): Book | undefined {
    return library?.books.find((book) => book.id === idea.bookId);
  }

  function setCompletionWorkDecision(workId: string, kind: CompletionWorkDecision["kind"], decision: string) {
    completionWorkDecisions = [
      ...completionWorkDecisions.filter((item) => item.workId !== workId || item.kind !== kind),
      { workId, kind, decision },
    ];
  }

  function hasCompletionDecisions(kind: "experiment" | "other"): boolean {
    const items = completionWorkItems.filter((item) =>
      kind === "experiment" ? item.kind === "experiment" : item.kind !== "experiment",
    );
    return items.every((item) =>
      completionWorkDecisions.some(
        (decision) => decision.workId === item.id && decision.kind === item.kind && decision.decision.trim(),
      ),
    );
  }
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

{#if context === "reader"}
  <ReaderView
    {library}
    {selectedBook}
    documentUrl={readerDocumentUrl}
    bind:page={readerPage}
    bind:zoom={readerZoom}
    bind:mode={readerMode}
    bind:images={readerImages}
    bind:sidebar={readerSidebar}
    bind:sidebarWidth={readerSidebarWidth}
    bind:search={readerSearch}
    bind:searchResults={readerSearchResults}
    {saveState}
    onChangeZoom={changeReaderZoom}
    onSetSidebar={setReaderSidebar}
    onCloseSidebar={closeReaderSidebar}
    {rememberSidebarTrigger}
    onSavePosition={saveReaderPosition}
    onPdfPosition={savePdfPosition}
    onSourceSelect={openSavedSource}
    onSearchResults={(results) => (readerSearchResults = results)}
    onSaveOutline={(outline) => {
      if (selectedBook && outline.length && selectedBook.outline.length === 0)
        void executeLibraryAction({ kind: "saveOutline", bookId: selectedBook.id, outline }, "");
    }}
    onSaveDraft={saveReaderDraft}
    onCreateIdea={createReaderIdea}
    onPersistPreferences={persistReaderPreferences}
  />
{:else}
  <div class="mx-auto max-w-[1500px] p-8 max-[1280px]:p-6">
    {#if feedback}<p role="status" class="mb-4 font-mono text-xs text-success">{feedback}</p>{/if}
    {#if loading}
      <div class="grid min-h-[60vh] place-items-center" role="status">Открываем личную библиотеку…</div>
    {:else if error && !library}
      <section class="mx-auto mt-24 max-w-xl rounded-xl border border-danger/40 bg-slate p-8 text-center" role="alert">
        <h2 class="text-xl font-semibold">Личная библиотека не открылась</h2>
        <p class="mt-3 text-mist-dim">{error}</p>
        <div class="mt-6"><Button onclick={() => location.reload()}>Повторить открытие</Button></div>
      </section>
    {:else if library}
      {#if error}<p class="mb-4 rounded-lg border border-danger/40 bg-danger/10 p-3 text-sm" role="alert">
          {error}
        </p>{/if}
      {#if context === "dashboard"}<DashboardView
          {library}
          {activeBook}
          {unfinishedCount}
          {busy}
          onImport={importBook}
        />
      {:else if context === "library"}<LibraryView
          {library}
          books={filteredBooks}
          bind:filter={libraryFilter}
          bind:sort={librarySort}
          onImport={importBook}
          {bookStatus}
        />
      {:else if context === "book"}<BookView
          {library}
          {selectedBook}
          {bookStatus}
          onRun={executeLibraryAction}
          onDelete={() => (deleteBookOpen = true)}
        />
      {:else if context === "drafts"}<DraftsView
          {library}
          {focusedDraft}
          {busy}
          onSelectDraft={(draftId) => (selectedDraftId = draftId)}
          onResolve={resolveDraft}
          onAttach={attachFocusedDraft}
          onRun={executeLibraryAction}
          onExport={exportFocusedDraft}
        />
      {:else if context === "knowledge" || context === "idea"}<KnowledgeView
          {library}
          {selectedIdea}
          bind:selectedTopic
          {bookForIdea}
          onSave={saveIdea}
          onLink={linkSelectedIdea}
          onPrepareReview={prepareIdeaReview}
        />
      {:else if context === "practice"}<PracticeView
          {library}
          onCompleteRecall={completeRecall}
          onRescheduleRecall={rescheduleRecall}
          onStartRecallNow={startRecallNow}
          onCreateExperiment={createExperiment}
          onAdvanceExperiment={advanceExperiment}
        />
      {:else if context === "completion"}<CompletionView
          step={completionStep}
          {library}
          {selectedBook}
          bind:significantIdeas
          bind:retrospective
          bind:unfinishedWorkDecision
          bind:continuingWork
          workItems={completionWorkItems}
          workDecisions={completionWorkDecisions}
          onCompleteReading={async () => {
            if (selectedBook)
              await executeLibraryAction({ kind: "completeReading", bookId: selectedBook.id }, "Чтение завершено");
            await saveCompletionStep(2);
          }}
          onSaveStep={saveCompletionStep}
          onSetWorkDecision={setCompletionWorkDecision}
          hasDecisions={hasCompletionDecisions}
          onFinish={finishStudy}
        />
      {:else if context === "settings"}<SettingsView
          {library}
          bind:section={settingsSection}
          bind:readerMode
          bind:readerImages
          {updateStatus}
          {diagnosticStatus}
          onRestoreBackup={restoreBackup}
          onExportArchive={exportArchive}
          onImportArchive={importArchive}
          onExportDiagnostics={exportDiagnostics}
          onCheckForUpdate={checkForUpdate}
          onPersistPreferences={persistReaderPreferences}
        />{/if}
    {/if}
  </div>

  <DialogModal
    bind:open={codexReviewOpen}
    title="Проверка идеи Codex"
    description="Проверьте минимальный пакет перед явной отправкой. PDF и другие записи не включены."
  >
    {#snippet trigger()}<span class="sr-only">Открыть проверку идеи</span>{/snippet}
    <TextArea id="codex-package" label="Подтверждаемый пакет" bind:value={codexPackage} />
    <Button variant="primary" disabled={!codexPackage.trim()} onclick={runIdeaReview}>Запустить проверку</Button>
    {#if pendingIdeaReview}<section class="grid gap-4 rounded-md border border-iris/25 bg-night/40 p-4">
        <div>
          <b>Обратная связь</b>
          <p class="mt-2 whitespace-pre-wrap text-sm leading-6">{pendingIdeaReview.response}</p>
        </div>
        <TextArea id="review-formulation" label="Авторская формулировка" bind:value={ideaFormulation} />
        <TextArea id="review-conclusion" label="Необязательный авторский вывод" bind:value={reviewConclusion} />
        <div class="flex flex-wrap gap-2">
          <Button variant="primary" disabled={!ideaFormulation.trim()} onclick={() => resolveIdeaReview("refined")}
            >Уточнить идею</Button
          ><Button onclick={() => resolveIdeaReview("unchanged")}>Оставить без изменений</Button><Button
            onclick={() => resolveIdeaReview("later")}>Вернуться позже</Button
          >
        </div>
      </section>{/if}
    {#if codexReviewResult}<p class="rounded-md border border-white/8 bg-night/40 p-4 text-sm" role="status">
        {codexReviewResult}
      </p>{/if}
  </DialogModal>
  <DialogModal
    bind:open={deleteBookOpen}
    title="Удалить книгу навсегда?"
    description="Это действие нельзя отменить. Проверьте, какие локальные данные исчезнут вместе с PDF."
  >
    {#snippet trigger()}<span class="sr-only">Подтвердить постоянное удаление книги</span>{/snippet}
    {#if selectedBook}<ul class="list-disc space-y-2 pl-5 text-sm text-mist-dim">
        <li>Сохранённый PDF «{selectedBook.title}»</li>
        <li>{library?.drafts.filter((draft) => draft.bookId === selectedBook.id).length ?? 0} черновых заметок</li>
        <li>{library?.ideas.filter((idea) => idea.bookId === selectedBook.id).length ?? 0} идей и их связей</li>
        <li>Связанные проверки, восстановления, эксперименты, вехи и черновик завершения</li>
      </ul>
      <div class="flex gap-2">
        <Button
          class="border-danger/40 text-danger hover:bg-danger/10"
          disabled={busy}
          onclick={permanentlyDeleteSelectedBook}>Удалить навсегда</Button
        ><Button onclick={() => (deleteBookOpen = false)}>Отмена</Button>
      </div>{/if}
  </DialogModal>
{/if}
