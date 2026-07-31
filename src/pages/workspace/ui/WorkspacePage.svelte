<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { Button, DialogModal } from "@/shared/ui";
  import {
    commandErrorMessage,
    type Book,
    type BackupMetadata,
    type ExperimentStatus,
    type Idea,
    type IdeaAssignment,
    type IdeaRelation,
    type LibraryAction,
    type ReviewDecision,
    type StudyCompletionDraft,
  } from "@/shared/api";
  import { useWorkspaceSession } from "../model/workspace-session.svelte";
  import type { WorkspaceContext } from "../model/workspace-context";
  import CompletionView from "./CompletionView.svelte";
  import DashboardView from "./DashboardView.svelte";
  import BookView from "./BookView.svelte";
  import DraftsView from "./DraftsView.svelte";
  import KnowledgeView from "./KnowledgeView.svelte";
  import LibraryView from "./LibraryView.svelte";
  import PracticeView from "./PracticeView.svelte";
  import ReaderWorkspace from "./ReaderWorkspace.svelte";
  import SettingsView from "./SettingsView.svelte";

  let {
    context,
    resourceId,
  }: {
    context: WorkspaceContext;
    resourceId?: string;
  } = $props();

  const session = useWorkspaceSession();
  const commands = $derived(session.commands);
  const library = $derived(session.library);
  const loading = $derived(session.loading);
  const busy = $derived(session.busy);
  const error = $derived(session.error);
  const feedback = $derived(session.feedback);

  let libraryFilter = $state("all");
  let librarySort = $state("recent");
  let selectedDraftId = $state("");
  let selectedTopic = $state("all");
  let bookFilterId = $state("");
  let deleteBookOpen = $state(false);
  let diagnosticEntries = $state<string[]>([]);
  let backupMetadata = $state<BackupMetadata>({ snapshotAt: null, archiveAt: null });

  let readerMode = $state<"muted" | "original" | "dark">("muted");
  let readerImages = $state(true);

  const activeBook = $derived(library?.books.find((book) => book.id === library?.activeStudyBookId) ?? null);
  const selectedBook = $derived(
    library?.books.find((book) => book.id === resourceId) ?? activeBook ?? library?.books[0] ?? null,
  );
  const selectedIdea = $derived(
    library?.ideas.find((idea) => idea.id === resourceId) ??
      library?.ideas.find((idea) => !bookFilterId || idea.bookId === bookFilterId) ??
      null,
  );
  const focusedDraft = $derived(
    library?.drafts.find((draft) => draft.id === selectedDraftId) ??
      library?.drafts.find((draft) => !bookFilterId || draft.bookId === bookFilterId) ??
      null,
  );
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
      await session.load();
      const loadedCommands = session.commands;
      if (!loadedCommands || !session.library) return;
      backupMetadata = await loadedCommands.backupMetadata();
      const query = new URLSearchParams(location.search);
      selectedDraftId = query.get("draft") ?? "";
      selectedTopic = query.get("topic") ?? "all";
      bookFilterId = query.get("book") ?? "";
      if (selectedBook) {
        readerMode =
          selectedBook.reader.documentMode === "mutedLight"
            ? "muted"
            : selectedBook.reader.documentMode === "darkInverted"
              ? "dark"
              : "original";
        readerImages = selectedBook.reader.invertImages;
      }
    } catch (cause) {
      session.error = commandErrorMessage(cause);
      recordDiagnostic("library-load", session.error);
    }
  });

  async function executeLibraryAction(action: LibraryAction, message = "Сохранено"): Promise<boolean> {
    const saved = await session.execute(action, message);
    if (!saved && session.error) recordDiagnostic(`action-${action.kind}`, session.error);
    return saved;
  }

  async function importBook() {
    if (!commands) return;
    session.busy = true;
    try {
      const result = await commands.importPdf();
      if (result) {
        session.replaceLibrary(
          result.state,
          result.duplicate ? "Эта редакция PDF уже есть в библиотеке" : "PDF импортирован",
        );
        await goto(resolve("/library/[bookId]", { bookId: result.bookId }));
      }
    } catch (cause) {
      session.error = commandErrorMessage(cause);
    } finally {
      session.busy = false;
    }
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
          sidebarOpen: selectedBook.reader.sidebarOpen,
          sidebarTab: selectedBook.reader.sidebarTab,
          sidebarWidth: selectedBook.reader.sidebarWidth,
        },
      },
      "",
    );
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
        session.replaceLibrary(snapshot, "Черновая заметка экспортирована");
      }
    } catch (cause) {
      session.error = commandErrorMessage(cause);
    }
  }

  async function prepareIdeaReview(): Promise<string> {
    if (!selectedIdea || !commands) return "";
    try {
      return await commands.prepareReview(selectedIdea.id, "ideaReview");
    } catch (cause) {
      session.error = commandErrorMessage(cause);
      return "";
    }
  }

  async function runIdeaReview(approvedPackage: string): Promise<string> {
    if (!selectedIdea || !commands || !approvedPackage) return "Пакет проверки пуст";
    try {
      session.replaceLibrary(await commands.runReview(selectedIdea.id, "ideaReview", approvedPackage));
      return "Проверка завершена. Ответ сохранён как ожидающее вашего решения замечание.";
    } catch (cause) {
      return commandErrorMessage(cause);
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

  async function resolveIdeaReview(
    decision: Exclude<ReviewDecision, "pending">,
    formulation: string,
    conclusion: string,
  ): Promise<boolean> {
    if (!selectedIdea) return false;
    return executeLibraryAction(
      {
        kind: "resolveReview",
        ideaId: selectedIdea.id,
        requestKind: "ideaReview",
        decision,
        formulation,
        conclusion,
      },
      decision === "refined"
        ? "Уточнённая формулировка сохранена"
        : decision === "unchanged"
          ? "Идея оставлена без изменений"
          : "Проверка сохранена на потом",
    );
  }

  async function permanentlyDeleteSelectedBook() {
    if (!selectedBook || !commands) return;
    session.busy = true;
    try {
      session.replaceLibrary(await commands.deleteBook(selectedBook.id));
      deleteBookOpen = false;
      await goto(resolve("/library"));
    } catch (cause) {
      session.error = commandErrorMessage(cause);
    } finally {
      session.busy = false;
    }
  }

  async function completeRecall(recallId: string, answer: string, rating: "confident" | "partial" | "notRecalled") {
    if (!recallId || !answer.trim()) return;
    await executeLibraryAction(
      { kind: "completeRecall", recallId, answer, rating },
      "Решение восстановления сохранено",
    );
  }

  async function rescheduleRecall(recallId: string, days: number) {
    const recall = library?.recalls.find((item) => item.id === recallId);
    if (!recall) return;
    await executeLibraryAction(
      { kind: "rescheduleRecall", recallId: recall.id, nextAt: recall.nextAt + days * 86_400 },
      `Следующее восстановление перенесено на ${days} дней`,
    );
  }

  async function startRecallNow(recallId: string) {
    const recall = library?.recalls.find((item) => item.id === recallId);
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

  async function saveExperimentDraft(draft: {
    id: string;
    ideaId: string;
    situation: string;
    action: string;
    nextStep: string;
  }) {
    await executeLibraryAction({ kind: "saveExperimentDraft", draft }, "Черновик эксперимента сохранён");
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

  async function saveCompletionStep(draft: StudyCompletionDraft) {
    await executeLibraryAction(
      {
        kind: "saveStudyCompletionDraft",
        draft,
      },
      "Черновик итога сохранён",
    );
  }

  async function finishStudy(draft: StudyCompletionDraft) {
    if (!selectedBook || draft.bookId !== selectedBook.id) return;
    await executeLibraryAction(
      {
        kind: "completeStudy",
        bookId: selectedBook.id,
        retrospective: draft.retrospective,
        significantIdeaIds: draft.significantIdeaIds,
        continuingWork: draft.continuingWork,
        unfinishedWorkDecision: draft.unfinishedWorkDecision,
        workDecisions: draft.workDecisions,
      },
      "Изучение завершено; продолжающаяся работа сохранена",
    );
  }

  async function restoreBackup(): Promise<string> {
    if (!commands) return "Команды библиотеки недоступны";
    try {
      session.replaceLibrary(await commands.restoreBackup());
      return "Последний snapshot восстановлен";
    } catch (cause) {
      return commandErrorMessage(cause);
    }
  }

  async function exportArchive(password: string): Promise<string> {
    if (!commands) return "Команды библиотеки недоступны";
    try {
      if (!(await commands.exportArchive(password))) return "Экспорт отменён";
      backupMetadata = await commands.backupMetadata();
      return "Переносимый архив сохранён";
    } catch (cause) {
      return commandErrorMessage(cause);
    }
  }

  async function importArchive(password: string): Promise<string> {
    if (!commands) return "Команды библиотеки недоступны";
    try {
      const snapshot = await commands.importArchive(password);
      if (snapshot) session.replaceLibrary(snapshot);
      return snapshot ? "Архив импортирован" : "Импорт отменён";
    } catch (cause) {
      return commandErrorMessage(cause);
    }
  }

  function recordDiagnostic(scope: string, message: string) {
    diagnosticEntries = [...diagnosticEntries, `${new Date().toISOString()} ${scope}: ${message}`].slice(-100);
  }

  async function checkForUpdate(): Promise<string> {
    if (!commands) return "Команды библиотеки недоступны";
    try {
      return (await commands.checkForUpdate())
        ? "Обновление установлено; перезапустите Bookshelf"
        : "Установлена актуальная версия";
    } catch (cause) {
      const message = commandErrorMessage(cause);
      recordDiagnostic("signed-update", message);
      return message;
    }
  }

  async function exportDiagnostics(): Promise<string> {
    if (!commands) return "Команды библиотеки недоступны";
    try {
      return (await commands.exportDiagnostics(diagnosticEntries))
        ? "Диагностический журнал экспортирован"
        : "Экспорт журнала отменён";
    } catch (cause) {
      return commandErrorMessage(cause);
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
</script>

{#if context === "reader"}
  <ReaderWorkspace {selectedBook} />
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
          onActivate={async () => {
            if (selectedBook)
              await executeLibraryAction({ kind: "activateStudy", bookId: selectedBook.id }, "Книга стала активной");
          }}
          onArchive={async () => {
            if (selectedBook)
              await executeLibraryAction({ kind: "archiveBook", bookId: selectedBook.id }, "Книга перемещена в архив");
          }}
          onRestore={async () => {
            if (selectedBook)
              await executeLibraryAction(
                { kind: "restoreBook", bookId: selectedBook.id },
                "Книга возвращена из архива",
              );
          }}
          onRepeat={async () => {
            if (selectedBook)
              await executeLibraryAction(
                { kind: "startRepeatStudy", bookId: selectedBook.id },
                "Начат новый цикл изучения",
              );
          }}
          onDelete={() => (deleteBookOpen = true)}
        />
      {:else if context === "drafts"}<DraftsView
          {library}
          {focusedDraft}
          {bookFilterId}
          {busy}
          onSelectDraft={(draftId) => (selectedDraftId = draftId)}
          onResolve={resolveDraft}
          onAttach={attachFocusedDraft}
          onDefer={async () => {
            if (focusedDraft)
              await executeLibraryAction({ kind: "deferDraft", draftId: focusedDraft.id }, "Заметка отложена");
          }}
          onDiscard={async () => {
            if (focusedDraft)
              await executeLibraryAction(
                { kind: "discardDraft", draftId: focusedDraft.id },
                "Черновая заметка удалена",
              );
          }}
          onExport={exportFocusedDraft}
        />
      {:else if context === "knowledge" || context === "idea"}<KnowledgeView
          {library}
          {selectedIdea}
          {bookFilterId}
          bind:selectedTopic
          {bookForIdea}
          onSave={saveIdea}
          onLink={linkSelectedIdea}
          onPrepareReview={prepareIdeaReview}
          onRunReview={runIdeaReview}
          onResolveReview={resolveIdeaReview}
        />
      {:else if context === "practice"}<PracticeView
          {library}
          {bookFilterId}
          onCompleteRecall={completeRecall}
          onRescheduleRecall={rescheduleRecall}
          onStartRecallNow={startRecallNow}
          onCreateExperiment={createExperiment}
          onSaveExperimentDraft={saveExperimentDraft}
          onAdvanceExperiment={advanceExperiment}
        />
      {:else if context === "completion"}<CompletionView
          {library}
          {selectedBook}
          onCompleteReading={async () => {
            if (selectedBook)
              await executeLibraryAction({ kind: "completeReading", bookId: selectedBook.id }, "Чтение завершено");
          }}
          onSaveStep={saveCompletionStep}
          onFinish={finishStudy}
        />
      {:else if context === "settings"}<SettingsView
          {library}
          {backupMetadata}
          bind:readerMode
          bind:readerImages
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
