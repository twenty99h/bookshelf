<script lang="ts">
  import { onMount } from "svelte";
  import { Button, CheckboxField, Eyebrow, SelectField, Surface, TextArea } from "@/shared/ui";
  import {
    commandErrorMessage,
    type Idea,
    type IdeaAssignment,
    type IdeaRelation,
    type LibraryAction,
    type LibraryState,
    type ReviewKind,
  } from "@/shared/api";
  import CodexReviewPanel from "./CodexReviewPanel.svelte";
  import ExperimentForm from "./ExperimentForm.svelte";
  import MaterialForm from "./MaterialForm.svelte";
  import RecallPanel from "./RecallPanel.svelte";
  import type { LibraryCommands, StopListening } from "../model/library-commands";

  const reviewCopy: Record<ReviewKind, { title: string }> = {
    ideaReview: { title: "Проверка идеи" },
    recallGaps: { title: "Проверка пробелов ответа" },
    topicSuggestion: { title: "Предложение темы" },
    linkSuggestion: { title: "Предложение связи" },
  };
  const workCardClass =
    "grid content-start gap-1.5 rounded-[11px] border border-rule bg-paper-raised p-6 shadow-paper [&_h2]:mb-2 [&_h2]:font-display [&_h2]:text-[25px] [&_h2]:font-medium [&_h2]:leading-tight [&_p]:mt-0 [&_label]:mb-1.5 [&_label]:block [&_label]:text-xs [&_label]:font-bold [&_label]:text-[#4d5861] [&_input]:w-full [&_input]:min-w-0 [&_input]:rounded-lg [&_input]:border [&_input]:border-[#cfd1cd] [&_input]:bg-paper-raised [&_input]:px-3 [&_input]:py-2.5 [&_input]:text-ink [&_input]:outline-none [&_textarea]:min-h-20 [&_textarea]:w-full [&_textarea]:min-w-0 [&_textarea]:resize-y [&_textarea]:rounded-lg [&_textarea]:border [&_textarea]:border-[#cfd1cd] [&_textarea]:bg-paper-raised [&_textarea]:px-3 [&_textarea]:py-2.5 [&_textarea]:text-ink [&_textarea]:outline-none [&_select]:w-full [&_select]:min-w-0 [&_select]:rounded-lg [&_select]:border [&_select]:border-[#cfd1cd] [&_select]:bg-paper-raised [&_select]:px-3 [&_select]:py-2.5 [&_select]:text-ink [&_select]:outline-none [&_input:focus]:border-leaf [&_input:focus]:ring-3 [&_input:focus]:ring-focus [&_textarea:focus]:border-leaf [&_textarea:focus]:ring-3 [&_textarea:focus]:ring-focus [&_select:focus]:border-leaf [&_select:focus]:ring-3 [&_select:focus]:ring-focus [&_fieldset]:my-1.5 [&_fieldset]:rounded-lg [&_fieldset]:border [&_fieldset]:border-rule [&_fieldset]:p-3 [&_legend]:px-1.5 [&_legend]:text-xs [&_legend]:font-bold [&_legend]:text-[#4d5861]";

  let {
    library,
    commands,
    run,
    bookTitle,
    mutate,
  }: {
    library: LibraryState;
    commands: LibraryCommands;
    run: (action: LibraryAction, success?: string) => Promise<void>;
    bookTitle: (bookId: string) => string;
    mutate: (command: () => Promise<LibraryState>, success: string) => Promise<LibraryState>;
  } = $props();
  let selectedId = $state("");
  let formulation = $state("");
  let recallAssigned = $state(true);
  let transferAssigned = $state(false);
  let experimentAssigned = $state(false);
  let masteredAssigned = $state(false);
  let topicId = $state("");
  let linkedIdeaId = $state("");
  let relation = $state<IdeaRelation>("complements");
  let recallIdea = $state<Idea | null>(null);
  let reviewIdea = $state<Idea | null>(null);
  let reviewKind = $state<ReviewKind>("ideaReview");
  let reviewPackageText = $state("");
  let reviewRecallAnswer = $state("");
  let reviewRequestId = $state("");
  let reviewResponse = $state("");
  let reviewError = $state("");
  let reviewRunning = $state(false);
  let reviewConclusion = $state("");
  let proposedTopic = $state("");
  let eventUnlisten: StopListening | undefined;

  let selected = $derived(library.ideas.find((idea) => idea.id === selectedId));
  let assignments = $derived(
    [
      recallAssigned && "recall",
      transferAssigned && "transfer",
      experimentAssigned && "experiment",
      masteredAssigned && "mastered",
    ].filter((value): value is IdeaAssignment => Boolean(value)),
  );
  let scheduledRecalls = $derived.by(() => {
    return library.recalls
      .filter(
        (recall, index, recalls) => recalls.findLastIndex((candidate) => candidate.ideaId === recall.ideaId) === index,
      )
      .toSorted((left, right) => left.nextAt - right.nextAt);
  });
  function selectIdea(idea: Idea) {
    selectedId = idea.id;
    formulation = idea.formulation;
    recallAssigned = idea.assignments.includes("recall");
    transferAssigned = idea.assignments.includes("transfer");
    experimentAssigned = idea.assignments.includes("experiment");
    masteredAssigned = idea.assignments.includes("mastered");
  }
  function startRecall(idea: Idea) {
    recallIdea = idea;
    reviewIdea = null;
  }
  async function saveIdea(event: SubmitEvent) {
    event.preventDefault();
    if (!selected) return;
    await run(
      { kind: "updateIdea", ideaId: selected.id, formulation, assignments },
      "Новая версия формулировки сохранена",
    );
  }
  async function completeExperiment(result: {
    situation: string;
    action: string;
    result: string;
    conclusion: string;
    successful: boolean;
  }) {
    if (!selected) return;
    await run(
      {
        kind: "completeExperiment",
        ideaId: selected.id,
        ...result,
      },
      "Практический эксперимент завершён",
    );
  }
  async function saveMaterial(material: {
    title: string;
    problem: string;
    idea: string;
    example: string;
    result: string;
    limitations: string;
  }) {
    if (!selected) return;
    await run(
      {
        kind: "saveMaterial",
        ...material,
        ideaIds: [selected.id],
      },
      "Материал для передачи сохранён",
    );
  }
  async function prepareReview(idea: Idea, kind: ReviewKind = "ideaReview", recallAnswer = "") {
    if (kind === "ideaReview" && selectedId !== idea.id) selectIdea(idea);
    reviewIdea = idea;
    reviewKind = kind;
    reviewRecallAnswer = recallAnswer;
    reviewPackageText = "";
    reviewResponse =
      library.reviews.find((item) => item.ideaId === idea.id && item.pending && item.requestKind === kind)?.response ??
      "";
    reviewError = "";
    try {
      reviewPackageText = await commands.prepareCodexReview(idea.id, kind, recallAnswer || undefined);
    } catch (cause) {
      reviewError = commandErrorMessage(cause);
    }
  }
  async function startReview() {
    if (!reviewIdea || reviewRunning) return;
    const idea = reviewIdea;
    reviewRequestId = crypto.randomUUID();
    reviewResponse = "";
    reviewError = "";
    reviewRunning = true;
    try {
      await mutate(
        () =>
          commands.runCodexReview(
            reviewRequestId,
            idea.id,
            reviewKind,
            reviewPackageText,
            reviewRecallAnswer || undefined,
          ),
        "Ответ Codex получен",
      );
    } catch (cause) {
      reviewError = commandErrorMessage(cause);
    } finally {
      reviewRunning = false;
    }
  }
  async function resolveCurrentReview(decision: "refined" | "unchanged" | "later", authoredFormulation = "") {
    if (!reviewIdea) return;
    await run(
      {
        kind: "resolveReview",
        ideaId: reviewIdea.id,
        requestKind: reviewKind,
        decision,
        formulation: authoredFormulation,
        conclusion: reviewConclusion,
      },
      decision === "later"
        ? "Проверка оставлена в долге изучения"
        : "Решение по проверке сохранено; полный ответ удалён",
    );
    if (decision !== "later") {
      reviewIdea = null;
      reviewResponse = "";
      reviewPackageText = "";
    }
  }
  async function confirmTopicSuggestion() {
    if (!reviewIdea || !proposedTopic.trim()) return;
    await run(
      { kind: "confirmSuggestedTopic", ideaId: reviewIdea.id, name: proposedTopic },
      "Предложенная тема создана и назначена идее",
    );
    await resolveCurrentReview("unchanged");
    proposedTopic = "";
  }
  async function confirmLink() {
    const source = reviewKind === "linkSuggestion" && reviewIdea ? reviewIdea : selected;
    if (!source || !linkedIdeaId) return;
    await run(
      { kind: "linkIdeas", fromIdeaId: source.id, toIdeaId: linkedIdeaId, relation },
      "Связь идей подтверждена",
    );
    if (reviewKind === "linkSuggestion" && reviewIdea?.id === source.id) await resolveCurrentReview("unchanged");
  }
  onMount(() => {
    void commands
      .onCodexReview((event) => {
        if (event.requestId === reviewRequestId && event.kind === "delta") reviewResponse += event.text;
      })
      .then((stop) => {
        eventUnlisten = stop;
      });
    return () => eventUnlisten?.();
  });
</script>

{#if recallIdea}
  <RecallPanel
    idea={recallIdea}
    {library}
    {run}
    onReview={(answer) => prepareReview(recallIdea!, "recallGaps", answer)}
    onClose={() => {
      recallIdea = null;
      reviewIdea = null;
    }}
  />
  {#if reviewIdea?.id === recallIdea.id}<CodexReviewPanel
      kind={reviewKind}
      title={reviewCopy[reviewKind].title}
      packageText={reviewPackageText}
      response={reviewResponse}
      error={reviewError}
      running={reviewRunning}
      authoredFormulation={formulation}
      bind:conclusion={reviewConclusion}
      bind:proposedTopic
      linkIdeas={library.ideas.filter((idea) => idea.id !== reviewIdea?.id)}
      bind:linkIdeaId={linkedIdeaId}
      bind:linkRelation={relation}
      onStart={startReview}
      onCancel={() => commands.cancelCodexReview(reviewRequestId)}
      onConfirmTopic={confirmTopicSuggestion}
      onConfirmLink={confirmLink}
      onReject={() => resolveCurrentReview("unchanged")}
      onRefine={() => resolveCurrentReview("refined", formulation)}
      onUnchanged={() => resolveCurrentReview("unchanged")}
      onLater={() => resolveCurrentReview("later")}
    />{/if}
{:else}
  {#if scheduledRecalls.length > 0}
    <section class="mb-5 grid gap-3" aria-label="Запланированные восстановления">
      <h2>Запланированные восстановления</h2>
      {#each scheduledRecalls as recall (recall.id)}
        {@const idea = library.ideas.find((candidate) => candidate.id === recall.ideaId)}
        {#if idea}<Surface
            ><div class="flex flex-wrap items-center justify-between gap-3">
              <div>
                <Eyebrow>Следующий срок</Eyebrow><b>{new Date(recall.nextAt * 1000).toLocaleDateString("ru")}</b>
                <p class="mb-0 mt-1">{idea.formulation}</p>
              </div>
              <div class="flex flex-wrap gap-2">
                <Button onclick={() => startRecall(idea)}>Начать сейчас</Button><Button
                  onclick={() =>
                    run(
                      {
                        kind: "rescheduleRecall",
                        recallId: recall.id,
                        nextAt: Math.floor(Date.now() / 1000) + 7 * 86_400,
                      },
                      "Восстановление перенесено на неделю",
                    )}>Перенести на неделю</Button
                >
              </div>
            </div></Surface
          >{/if}
      {/each}
    </section>
  {/if}
  <section
    class="grid gap-3.5 [&_blockquote]:my-3.5 [&_blockquote]:border-l-[3px] [&_blockquote]:border-[#b8d94a] [&_blockquote]:py-1.5 [&_blockquote]:pl-4 [&_blockquote]:leading-relaxed [&_blockquote]:text-[#45515a] [&_details]:mt-3.5 [&_details]:border-t [&_details]:border-rule [&_details]:pt-3 [&_h2]:mb-2 [&_h2]:font-display [&_h2]:text-[25px] [&_h2]:font-medium [&_h2]:leading-tight [&_p]:mt-0 [&_summary]:cursor-pointer [&_summary]:font-bold"
  >
    {#each library.ideas as idea (idea.id)}
      {@const pendingReviews = library.reviews.filter((review) => review.ideaId === idea.id && review.pending)}
      <Surface
        ><Eyebrow>{bookTitle(idea.bookId)} · {idea.section}</Eyebrow>
        <h2>{idea.formulation}</h2>
        <p>Назначения: {idea.assignments.join(", ")}</p>
        {#if pendingReviews.length}<p class="rounded-lg bg-leaf-soft px-3 py-2 text-sm">
            <b>В долге:</b>
            {pendingReviews.length} отложенных проверок ждут решения.
          </p>{/if}
        <div class="mt-2.5 flex flex-wrap items-center gap-2">
          <Button onclick={() => selectIdea(idea)}>Развить идею</Button><Button onclick={() => startRecall(idea)}
            >Восстановить знание</Button
          >{#each pendingReviews as pendingReview (pendingReview.id)}<Button
              variant="primary"
              onclick={() => prepareReview(idea, pendingReview.requestKind)}
              >Разобрать: {reviewCopy[pendingReview.requestKind].title}</Button
            >{/each}{#if pendingReviews.length === 0}<Button onclick={() => prepareReview(idea)}>Проверить идею</Button
            >{/if}
        </div>
        <details>
          <summary>Источник и история</summary
          >{#each idea.fragments as fragment (`${fragment.page}-${fragment.excerpt}-${fragment.context}`)}<blockquote>
              стр. {fragment.page}: {fragment.excerpt}
            </blockquote>{/each}
          {#each idea.versions as version (`${version.savedAt}-${version.formulation}`)}<p>
              {new Date(version.savedAt * 1000).toLocaleString("ru")}: {version.formulation}
            </p>{/each}
        </details></Surface
      >
    {/each}
  </section>

  {#if selected}
    <section class="mt-5 grid grid-cols-1 gap-4 sm:grid-cols-2">
      <form class={workCardClass} onsubmit={saveIdea}>
        <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">
          Авторская формулировка
        </p>
        <h2>Развить идею</h2>
        <TextArea id="formulation" label="Текущая формулировка" bind:value={formulation} />
        <fieldset>
          <legend>Назначения идеи</legend>
          <div class="grid gap-2 py-2">
            <CheckboxField label="Восстановление" bind:checked={recallAssigned} />
            <CheckboxField label="Передача" bind:checked={transferAssigned} />
            <CheckboxField label="Практический эксперимент" bind:checked={experimentAssigned} />
            <CheckboxField label="Уже освоено" bind:checked={masteredAssigned} />
          </div>
        </fieldset>
        <Button type="submit">Сохранить новую версию</Button>
      </form>

      <form
        class={workCardClass}
        onsubmit={(event) => {
          event.preventDefault();
          if (topicId) run({ kind: "assignTopic", ideaId: selected!.id, topicId }, "Идея добавлена в тему");
        }}
      >
        <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">Организация знаний</p>
        <h2>Тема и связь</h2>
        <SelectField
          label="Тема"
          bind:value={topicId}
          placeholder="Выберите тему"
          options={library.topics.map((topic) => ({ value: topic.id, label: topic.name }))}
        />
        <Button type="submit">Добавить в тему</Button>
        <SelectField
          label="Связанная идея"
          bind:value={linkedIdeaId}
          placeholder="Выберите идею"
          options={library.ideas
            .filter((idea) => idea.id !== selected?.id)
            .map((idea) => ({ value: idea.id, label: idea.formulation }))}
        />
        <SelectField
          label="Тип связи"
          bind:value={relation}
          options={[
            { value: "complements", label: "Дополняет" },
            { value: "clarifies", label: "Уточняет" },
            { value: "contradicts", label: "Противоречит" },
          ]}
        />
        <Button onclick={confirmLink}>Подтвердить связь</Button>
      </form>

      <ExperimentForm onComplete={completeExperiment} />
      <MaterialForm onSave={saveMaterial} />
    </section>
    <div class="my-[18px] flex flex-wrap items-center gap-2">
      <Button onclick={() => prepareReview(selected!, "topicSuggestion")}>Предложить тему через Codex</Button><Button
        onclick={() => prepareReview(selected!, "linkSuggestion")}>Предложить дубль или связь</Button
      >
    </div>
  {/if}
  {#if reviewIdea}<CodexReviewPanel
      kind={reviewKind}
      title={reviewCopy[reviewKind].title}
      packageText={reviewPackageText}
      response={reviewResponse}
      error={reviewError}
      running={reviewRunning}
      authoredFormulation={formulation}
      bind:conclusion={reviewConclusion}
      bind:proposedTopic
      linkIdeas={library.ideas.filter((idea) => idea.id !== reviewIdea?.id)}
      bind:linkIdeaId={linkedIdeaId}
      bind:linkRelation={relation}
      onStart={startReview}
      onCancel={() => commands.cancelCodexReview(reviewRequestId)}
      onConfirmTopic={confirmTopicSuggestion}
      onConfirmLink={confirmLink}
      onReject={() => resolveCurrentReview("unchanged")}
      onRefine={() => resolveCurrentReview("refined", formulation)}
      onUnchanged={() => resolveCurrentReview("unchanged")}
      onLater={() => resolveCurrentReview("later")}
    />{/if}
{/if}
