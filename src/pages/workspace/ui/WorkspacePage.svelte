<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { BookCopy, Brain, Command, FlaskConical, Gauge, Library, Search, Settings, StickyNote } from "@lucide/svelte";
  import { Button, DialogModal, TextArea, TextField } from "@/shared/ui";
  import {
    commandErrorMessage,
    type Book,
    type CompletionWorkDecision,
    type Idea,
    type IdeaAssignment,
    type IdeaRelation,
    type LibraryAction,
    type LibraryState,
    type ReviewDecision,
    type SourceFragment,
  } from "@/shared/api";
  import { createWorkspaceCommands, type WorkspaceCommands, type WorkspaceContext } from "@/features/workspace";
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
  let paletteOpen = $state(false);
  let paletteQuery = $state("");
  let paletteResults = $state<{ id: string; kind: string; title: string; context: string }[]>([]);

  let libraryFilter = $state("all");
  let librarySort = $state("recent");
  let draftMode = $state<"focus" | "list">("focus");
  let selectedDraftId = $state("");
  let draftFormulation = $state("");
  let selectedTopic = $state("all");
  let recallAnswer = $state("");
  let recallRevealed = $state(false);
  let experimentStep = $state("running");
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
  let ideaAssignments = $state<IdeaAssignment[]>([]);
  let relatedIdeaId = $state("");
  let ideaRelation = $state<IdeaRelation>("complements");
  let reviewConclusion = $state("");
  let deleteBookOpen = $state(false);
  let backupPassword = $state("");
  let backupStatus = $state("");
  let settingsSection = $state<"interface" | "library" | "backups" | "ai">("interface");
  let updateStatus = $state("");
  let diagnosticStatus = $state("");
  let diagnosticEntries = $state<string[]>([]);
  let experimentNextStep = $state("");
  let experimentCancellationReason = $state("");

  let readerSidebar = $state<"note" | "outline" | "search" | null>(null);
  let readerSidebarWidth = $state(400);
  let readerZoom = $state(1.15);
  let readerPage = $state(286);
  let readerMode = $state("muted");
  let readerImages = $state(true);
  let readerSearch = $state("");
  let readerSearchResults = $state<{ page: number; excerpt: string }[]>([]);
  let readerExcerpt = $state("");
  let readerFragments = $state<SourceFragment[]>([]);
  let readerComment = $state("");
  let readerIdeaDraftId = $state("");
  let readerIdeaFormulation = $state("");
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
        ideaAssignments = [...selectedIdea.assignments];
        relatedIdeaId = library.ideas.find((idea) => idea.id !== selectedIdea.id)?.id ?? "";
      }
    } catch (cause) {
      error = commandErrorMessage(cause);
      recordDiagnostic("library-load", error);
    } finally {
      loading = false;
    }
  });

  async function run(action: LibraryAction, message = "Сохранено"): Promise<boolean> {
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

  async function searchPalette() {
    if (!commands) return;
    paletteResults = await commands.search(paletteQuery);
  }

  function openPaletteResult(result: { id: string; kind: string }) {
    paletteOpen = false;
    if (result.kind === "book") goto(resolve("/library/[bookId]", { bookId: result.id }));
    else if (result.kind === "idea") goto(resolve("/knowledge/[ideaId]", { ideaId: result.id }));
    else if (result.kind === "topic") goto(resolve(`/knowledge?topic=${encodeURIComponent(result.id)}`));
    else if (result.kind === "draft") goto(resolve(`/drafts?draft=${encodeURIComponent(result.id)}`));
    else if (result.kind === "material") goto(resolve(`/drafts?material=${encodeURIComponent(result.id)}`));
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

  async function saveReaderDraft(): Promise<string | null> {
    if (!selectedBook || !readerExcerpt.trim()) return null;
    const existingDraftIds = new Set(library?.drafts.map((draft) => draft.id) ?? []);
    const saved = await run(
      readerFragments.length
        ? {
            kind: "captureDraftSources",
            bookId: selectedBook.id,
            section: "Глава 5 · Репликация",
            fragments: readerFragments.map((fragment) => ({ ...fragment })),
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
    if (!saved) return null;
    const createdDraft = library?.drafts.find((draft) => !existingDraftIds.has(draft.id)) ?? null;
    readerExcerpt = "";
    readerFragments = [];
    readerComment = "";
    return createdDraft?.id ?? null;
  }

  async function startReaderIdea() {
    const draftId = await saveReaderDraft();
    if (draftId) readerIdeaDraftId = draftId;
  }

  async function createReaderIdea() {
    if (!readerIdeaDraftId || !readerIdeaFormulation.trim()) return;
    const draft = library?.drafts.find((item) => item.id === readerIdeaDraftId);
    if (!draft) return;
    const created = await run(
      {
        kind: "resolveDraftAsIdea",
        draftId: draft.id,
        formulation: readerIdeaFormulation,
        section: draft.section,
        assignments: [],
      },
      "Идея сохранена; учебное назначение можно выбрать позже",
    );
    if (created) {
      readerIdeaDraftId = "";
      readerIdeaFormulation = "";
    }
  }

  async function saveReaderPosition(page = readerPage) {
    if (!selectedBook) return;
    readerPage = page;
    saveState = "saving";
    saveState = (await run(
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

  function openSavedSource(source: SourceFragment) {
    readerPage = source.page;
    readerExcerpt = source.excerpt;
    readerFragments = [source];
    setReaderSidebar("note");
  }

  function capturePdfSelection(fragments: SourceFragment[]) {
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
    void run({ kind: "updateReading", bookId: selectedBook!.id, page, zoom: readerZoom, scroll }, "").then(
      (saved) => (saveState = saved ? "saved" : "error"),
    );
  }

  async function resolveDraft() {
    if (!focusedDraft || !draftFormulation.trim()) return;
    await run(
      {
        kind: "resolveDraftAsIdea",
        draftId: focusedDraft.id,
        formulation: draftFormulation,
        section: focusedDraft.section,
        assignments: [],
      },
      "Идея сформулирована; источник сохранён",
    );
    draftFormulation = "";
  }

  async function attachFocusedDraft() {
    const idea = library?.ideas.find((item) => item.bookId === focusedDraft?.bookId);
    if (!focusedDraft || !idea) return;
    await run({ kind: "attachDraftToIdea", draftId: focusedDraft.id, ideaId: idea.id }, "Источник присоединён к идее");
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

  async function saveIdea() {
    if (!selectedIdea || !ideaFormulation.trim() || ideaAssignments.length === 0) return;
    await run(
      {
        kind: "updateIdea",
        ideaId: selectedIdea.id,
        formulation: ideaFormulation,
        assignments: [...ideaAssignments],
      },
      "Идея и назначения сохранены",
    );
  }

  function toggleIdeaAssignment(assignment: IdeaAssignment, checked: boolean) {
    ideaAssignments = checked
      ? [...new Set([...ideaAssignments, assignment])]
      : ideaAssignments.filter((candidate) => candidate !== assignment);
  }

  async function linkSelectedIdea() {
    if (!selectedIdea || !relatedIdeaId) return;
    await run(
      { kind: "linkIdeas", fromIdeaId: selectedIdea.id, toIdeaId: relatedIdeaId, relation: ideaRelation },
      "Связь идей подтверждена",
    );
  }

  async function resolveIdeaReview(decision: Exclude<ReviewDecision, "pending">) {
    if (!selectedIdea) return;
    const saved = await run(
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

  async function completeRecall(rating: "confident" | "partial" | "notRecalled") {
    const idea = library?.ideas[0];
    if (!idea || !recallAnswer.trim()) return;
    await run(
      { kind: "completeRecall", ideaId: idea.id, answer: recallAnswer, rating },
      "Решение восстановления сохранено",
    );
  }

  async function advanceExperiment(status: "reviewing" | "completed" | "cancelled") {
    const experiment = library?.experiments[0];
    if (!experiment) return;
    await run(
      {
        kind: "advanceExperiment",
        experimentId: experiment.id,
        status,
        situation: experiment.situation,
        action: experiment.action,
        result: experiment.result,
        conclusion: experiment.conclusion,
        cancellationReason: experimentCancellationReason,
        nextStep: experimentNextStep || experiment.nextStep,
      },
      "Состояние эксперимента сохранено",
    );
    experimentStep = status === "reviewing" ? "review" : status;
  }

  async function saveCompletionStep(nextStep: number) {
    if (!selectedBook) return;
    await run(
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
    await run(
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

  async function restoreBackup() {
    if (!commands) return;
    try {
      backupStatus = "Восстановление…";
      library = await commands.restoreBackup();
      backupStatus = "Последний snapshot восстановлен";
    } catch (cause) {
      backupStatus = commandErrorMessage(cause);
    }
  }

  async function exportArchive() {
    if (!commands) return;
    try {
      backupStatus = "Экспорт…";
      backupStatus = (await commands.exportArchive(backupPassword)) ? "Переносимый архив сохранён" : "Экспорт отменён";
    } catch (cause) {
      backupStatus = commandErrorMessage(cause);
    }
  }

  async function importArchive() {
    if (!commands) return;
    try {
      backupStatus = "Импорт…";
      const snapshot = await commands.importArchive(backupPassword);
      if (snapshot) library = snapshot;
      backupStatus = snapshot ? "Архив импортирован" : "Импорт отменён";
    } catch (cause) {
      backupStatus = commandErrorMessage(cause);
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
    bind:excerpt={readerExcerpt}
    bind:comment={readerComment}
    bind:ideaDraftId={readerIdeaDraftId}
    bind:ideaFormulation={readerIdeaFormulation}
    {saveState}
    onChangeZoom={changeReaderZoom}
    onSetSidebar={setReaderSidebar}
    onCloseSidebar={closeReaderSidebar}
    {rememberSidebarTrigger}
    onSavePosition={saveReaderPosition}
    onPdfPosition={savePdfPosition}
    onSelection={capturePdfSelection}
    onSourceSelect={openSavedSource}
    onSearchResults={(results) => (readerSearchResults = results)}
    onSaveOutline={(outline) => {
      if (selectedBook && outline.length && selectedBook.outline.length === 0)
        void run({ kind: "saveOutline", bookId: selectedBook.id, outline }, "");
    }}
    onSaveDraft={saveReaderDraft}
    onStartIdea={startReaderIdea}
    onCreateIdea={createReaderIdea}
    onPersistPreferences={persistReaderPreferences}
  />
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
                onRun={run}
                onDelete={() => (deleteBookOpen = true)}
              />
            {:else if context === "drafts"}<DraftsView
                {library}
                {focusedDraft}
                bind:mode={draftMode}
                bind:formulation={draftFormulation}
                {busy}
                onResolve={resolveDraft}
                onAttach={attachFocusedDraft}
                onRun={run}
                onExport={exportFocusedDraft}
              />
            {:else if context === "knowledge" || context === "idea"}<KnowledgeView
                {library}
                {selectedIdea}
                bind:selectedTopic
                bind:formulation={ideaFormulation}
                bind:assignments={ideaAssignments}
                bind:relatedIdeaId
                bind:relation={ideaRelation}
                {bookForIdea}
                onToggleAssignment={toggleIdeaAssignment}
                onSave={saveIdea}
                onLink={linkSelectedIdea}
                onPrepareReview={prepareIdeaReview}
              />
            {:else if context === "practice"}<PracticeView
                {library}
                bind:recallAnswer
                bind:recallRevealed
                {experimentStep}
                bind:experimentNextStep
                bind:experimentCancellationReason
                onCompleteRecall={completeRecall}
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
                  if (selectedBook) await run({ kind: "completeReading", bookId: selectedBook.id }, "Чтение завершено");
                  await saveCompletionStep(2);
                }}
                onSaveStep={saveCompletionStep}
                onSetWorkDecision={setCompletionWorkDecision}
                hasDecisions={hasCompletionDecisions}
                onFinish={finishStudy}
              />
            {:else if context === "settings"}<SettingsView
                bind:section={settingsSection}
                bind:readerMode
                bind:readerImages
                bind:backupPassword
                {backupStatus}
                {updateStatus}
                {diagnosticStatus}
                onRestoreBackup={restoreBackup}
                onExportArchive={exportArchive}
                onImportArchive={importArchive}
                onExportDiagnostics={exportDiagnostics}
                onCheckForUpdate={checkForUpdate}
              />{/if}
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
