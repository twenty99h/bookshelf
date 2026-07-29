<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { Button, Eyebrow, Surface } from "@/shared/ui";
  import { cancelCodexReview, commandErrorMessage, runCodexReview, type CodexStreamEvent, type Idea, type LibraryAction, type LibraryState, type ReviewKind } from "@/shared/api";
  import CodexReviewPanel from "./CodexReviewPanel.svelte";
  import RecallPanel from "./RecallPanel.svelte";

  const reviewCopy: Record<ReviewKind, { title: string; question: string }> = {
    ideaReview: { title: "Проверка идеи", question: "При каких условиях моя формулировка неточна или неприменима?" },
    recallGaps: { title: "Проверка пробелов ответа", question: "Какие существенные пробелы есть в моём ответе без выставления самооценки?" },
    topicSuggestion: { title: "Предложение темы", question: "Предложи одну подходящую тему знаний и объясни связь." },
    linkSuggestion: { title: "Предложение связи", question: "Предложи ровно одну наиболее обоснованную смысловую связь с одной из перечисленных идей." },
  };

  let { library, run, bookTitle, onLibrary }: { library: LibraryState; run: (action: LibraryAction, success?: string) => Promise<void>; bookTitle: (bookId: string) => string; onLibrary: (state: LibraryState) => void } = $props();
  let selectedId = $state("");
  let formulation = $state("");
  let assignments = $state<string[]>(["recall"]);
  let topicId = $state("");
  let linkedIdeaId = $state("");
  let relation = $state("complements");
  let situation = $state("");
  let actionTaken = $state("");
  let observedResult = $state("");
  let conclusion = $state("");
  let successful = $state(false);
  let recallIdea = $state<Idea | null>(null);
  let materialTitle = $state("");
  let problem = $state("");
  let materialIdea = $state("");
  let example = $state("");
  let materialResult = $state("");
  let limitations = $state("");
  let reviewIdea = $state<Idea | null>(null);
  let reviewKind = $state<ReviewKind>("ideaReview");
  let reviewPackageText = $state("");
  let reviewRequestId = $state("");
  let reviewResponse = $state("");
  let reviewError = $state("");
  let reviewRunning = $state(false);
  let reviewConclusion = $state("");
  let proposedTopic = $state("");
  let eventUnlisten: UnlistenFn | undefined;

  let selected = $derived(library.ideas.find((idea) => idea.id === selectedId));
  let scheduledRecalls = $derived.by(() => {
    const latest = new Map<string, LibraryState["recalls"][number]>();
    for (const recall of library.recalls) latest.set(recall.ideaId, recall);
    return [...latest.values()].sort((left, right) => left.nextAt - right.nextAt);
  });
  function selectIdea(idea: Idea) { selectedId = idea.id; formulation = idea.formulation; assignments = [...idea.assignments]; }
  function startRecall(idea: Idea) { recallIdea = idea; reviewIdea = null; }
  async function saveIdea(event: SubmitEvent) { event.preventDefault(); if (!selected) return; await run({ kind: "updateIdea", ideaId: selected.id, formulation, assignments }, "Новая версия формулировки сохранена"); }
  async function completeExperiment(event: SubmitEvent) { event.preventDefault(); if (!selected) return; await run({ kind: "completeExperiment", ideaId: selected.id, situation, action: actionTaken, result: observedResult, conclusion, successful }, "Практический эксперимент завершён"); }
  async function saveMaterial(event: SubmitEvent) { event.preventDefault(); if (!selected) return; await run({ kind: "saveMaterial", title: materialTitle, problem, idea: materialIdea, example, result: materialResult, limitations, ideaIds: [selected.id] }, "Материал для передачи сохранён"); }
  function buildReviewPackage(idea: Idea, kind: ReviewKind, recallAnswer = "") {
    const fragment = idea.fragments[0];
    const related = kind === "linkSuggestion" ? `Кандидаты для сравнения: ${library.ideas.filter((candidate) => candidate.id !== idea.id).map((candidate) => candidate.formulation).join("; ") || "нет"}` : "";
    return [
      "Инструкция: укажи возможные пробелы и ограничения; не переписывай идею за читателя и не выставляй итоговую оценку.",
      `Источник: ${bookTitle(idea.bookId)}, ${idea.section}${fragment ? `, стр. ${fragment.page}` : ""}`,
      fragment ? `Выбранный фрагмент: ${fragment.excerpt}` : "",
      `Авторская формулировка: ${idea.formulation}`,
      kind === "recallGaps" ? `Ответ читателя: ${recallAnswer}` : "",
      related,
      `Вопрос: ${reviewCopy[kind].question}`,
      "Критерии ответа: точность, существенные ограничения, связь с показанным источником; никаких автоматических изменений.",
    ].filter(Boolean).join("\n\n");
  }
  function prepareReview(idea: Idea, kind: ReviewKind = "ideaReview", recallAnswer = "") {
    if (kind === "ideaReview" && selectedId !== idea.id) selectIdea(idea);
    reviewIdea = idea;
    reviewKind = kind;
    reviewPackageText = buildReviewPackage(idea, kind, recallAnswer);
    reviewResponse = library.reviews.find((item) => item.ideaId === idea.id && item.pending && item.requestKind === kind)?.response ?? "";
    reviewError = "";
  }
  async function startReview() {
    if (!reviewIdea || reviewRunning) return;
    reviewRequestId = crypto.randomUUID();
    reviewResponse = "";
    reviewError = "";
    reviewRunning = true;
    try {
      onLibrary(await runCodexReview(reviewRequestId, reviewIdea.id, reviewKind, reviewPackageText));
    } catch (cause) { reviewError = commandErrorMessage(cause); }
    finally { reviewRunning = false; }
  }
  async function resolveCurrentReview(decision: "refined" | "unchanged" | "later", authoredFormulation = "") {
    if (!reviewIdea) return;
    await run({ kind: "resolveReview", ideaId: reviewIdea.id, requestKind: reviewKind, decision, formulation: authoredFormulation, conclusion: reviewConclusion }, decision === "later" ? "Проверка оставлена в долге изучения" : "Решение по проверке сохранено; полный ответ удалён");
    if (decision !== "later") { reviewIdea = null; reviewResponse = ""; reviewPackageText = ""; }
  }
  async function confirmTopicSuggestion() {
    if (!reviewIdea || !proposedTopic.trim()) return;
    await run({ kind: "confirmSuggestedTopic", ideaId: reviewIdea.id, name: proposedTopic }, "Предложенная тема создана и назначена идее");
    await resolveCurrentReview("unchanged");
    proposedTopic = "";
  }
  async function confirmLink() {
    const source = reviewKind === "linkSuggestion" && reviewIdea ? reviewIdea : selected;
    if (!source || !linkedIdeaId) return;
    await run({ kind: "linkIdeas", fromIdeaId: source.id, toIdeaId: linkedIdeaId, relation }, "Связь идей подтверждена");
    if (reviewKind === "linkSuggestion" && reviewIdea?.id === source.id) await resolveCurrentReview("unchanged");
  }
  onMount(() => {
    void listen<CodexStreamEvent>("codex-review-event", (event) => {
      if (event.payload.requestId === reviewRequestId && event.payload.kind === "delta") reviewResponse += event.payload.text;
    }).then((stop) => { eventUnlisten = stop; });
    return () => eventUnlisten?.();
  });
</script>

{#if recallIdea}
  <RecallPanel idea={recallIdea} {library} {run} onReview={(answer) => prepareReview(recallIdea!, "recallGaps", answer)} onClose={() => { recallIdea = null; reviewIdea = null; }} />
    {#if reviewIdea?.id === recallIdea.id}<CodexReviewPanel kind={reviewKind} title={reviewCopy[reviewKind].title} packageText={reviewPackageText} response={reviewResponse} error={reviewError} running={reviewRunning} authoredFormulation={formulation} bind:conclusion={reviewConclusion} bind:proposedTopic linkIdeas={library.ideas.filter((idea) => idea.id !== reviewIdea?.id)} bind:linkIdeaId={linkedIdeaId} bind:linkRelation={relation} onStart={startReview} onCancel={() => cancelCodexReview(reviewRequestId)} onConfirmTopic={confirmTopicSuggestion} onConfirmLink={confirmLink} onReject={() => resolveCurrentReview("unchanged")} onRefine={() => resolveCurrentReview("refined", formulation)} onUnchanged={() => resolveCurrentReview("unchanged")} onLater={() => resolveCurrentReview("later")} />{/if}
{:else}
  {#if scheduledRecalls.length > 0}
    <section class="mb-5 grid gap-3" aria-label="Запланированные восстановления">
      <h2>Запланированные восстановления</h2>
      {#each scheduledRecalls as recall}
        {@const idea = library.ideas.find((candidate) => candidate.id === recall.ideaId)}
        {#if idea}<Surface><div class="flex flex-wrap items-center justify-between gap-3"><div><Eyebrow>Следующий срок</Eyebrow><b>{new Date(recall.nextAt * 1000).toLocaleDateString("ru")}</b><p class="mb-0 mt-1">{idea.formulation}</p></div><div class="flex flex-wrap gap-2"><Button onclick={() => startRecall(idea)}>Начать сейчас</Button><Button onclick={() => run({ kind: "rescheduleRecall", recallId: recall.id, nextAt: Math.floor(Date.now() / 1000) + 7 * 86_400 }, "Восстановление перенесено на неделю")}>Перенести на неделю</Button></div></div></Surface>{/if}
      {/each}
    </section>
  {/if}
  <section class="stack">
    {#each library.ideas as idea}
      {@const pendingReviews = library.reviews.filter((review) => review.ideaId === idea.id && review.pending)}
      <Surface><Eyebrow>{bookTitle(idea.bookId)} · {idea.section}</Eyebrow><h2>{idea.formulation}</h2><p>Назначения: {idea.assignments.join(", ")}</p>{#if pendingReviews.length}<p class="rounded-lg bg-leaf-soft px-3 py-2 text-sm"><b>В долге:</b> {pendingReviews.length} отложенных проверок ждут решения.</p>{/if}<div class="card-actions"><Button onclick={() => selectIdea(idea)}>Развить идею</Button><Button onclick={() => startRecall(idea)}>Восстановить знание</Button>{#each pendingReviews as pendingReview}<Button variant="primary" onclick={() => prepareReview(idea, pendingReview.requestKind)}>Разобрать: {reviewCopy[pendingReview.requestKind].title}</Button>{/each}{#if pendingReviews.length === 0}<Button onclick={() => prepareReview(idea)}>Проверить идею</Button>{/if}</div><details><summary>Источник и история</summary>{#each idea.fragments as fragment}<blockquote>стр. {fragment.page}: {fragment.excerpt}</blockquote>{/each}{#each idea.versions as version}<p>{new Date(version.savedAt * 1000).toLocaleString("ru")}: {version.formulation}</p>{/each}</details></Surface>
    {/each}
  </section>

  {#if selected}
    <section class="idea-workbench">
      <form class="work-card" onsubmit={saveIdea}><p class="eyebrow">Авторская формулировка</p><h2>Развить идею</h2><label for="formulation">Текущая формулировка</label><textarea id="formulation" bind:value={formulation}></textarea><fieldset><legend>Назначения идеи</legend><label class="checkbox"><input type="checkbox" value="recall" bind:group={assignments} /> Восстановление</label><label class="checkbox"><input type="checkbox" value="transfer" bind:group={assignments} /> Передача</label><label class="checkbox"><input type="checkbox" value="experiment" bind:group={assignments} /> Практический эксперимент</label><label class="checkbox"><input type="checkbox" value="mastered" bind:group={assignments} /> Уже освоено</label></fieldset><Button type="submit">Сохранить новую версию</Button></form>

      <form class="work-card" onsubmit={(event) => { event.preventDefault(); if (topicId) run({ kind: "assignTopic", ideaId: selected!.id, topicId }, "Идея добавлена в тему"); }}><p class="eyebrow">Организация знаний</p><h2>Тема и связь</h2><label for="topic">Тема</label><select id="topic" bind:value={topicId}><option value="">Выберите тему</option>{#each library.topics as topic}<option value={topic.id}>{topic.name}</option>{/each}</select><Button type="submit">Добавить в тему</Button><label for="linked">Связанная идея</label><select id="linked" bind:value={linkedIdeaId}><option value="">Выберите идею</option>{#each library.ideas.filter((idea) => idea.id !== selected?.id) as idea}<option value={idea.id}>{idea.formulation}</option>{/each}</select><select aria-label="Тип связи" bind:value={relation}><option value="complements">Дополняет</option><option value="clarifies">Уточняет</option><option value="contradicts">Противоречит</option></select><Button onclick={confirmLink}>Подтвердить связь</Button></form>

      <form class="work-card" onsubmit={completeExperiment}><p class="eyebrow">Практический эксперимент</p><h2>Зафиксировать результат</h2><label for="situation">Ситуация</label><textarea id="situation" bind:value={situation}></textarea><label for="action-taken">Действие</label><textarea id="action-taken" bind:value={actionTaken}></textarea><label for="observed">Наблюдаемый результат</label><textarea id="observed" bind:value={observedResult}></textarea><label for="conclusion">Мой вывод</label><textarea id="conclusion" bind:value={conclusion}></textarea><label class="checkbox"><input type="checkbox" bind:checked={successful} /> Результат оказался положительным</label><Button type="submit">Завершить эксперимент</Button></form>

      <form class="work-card" onsubmit={saveMaterial}><p class="eyebrow">Передача знания</p><h2>Авторский Markdown-материал</h2><label for="material-title">Название</label><input id="material-title" bind:value={materialTitle} /><label for="problem">Проблема</label><textarea id="problem" bind:value={problem}></textarea><label for="material-idea">Идея</label><textarea id="material-idea" bind:value={materialIdea}></textarea><label for="example">Пример применения</label><textarea id="example" bind:value={example}></textarea><label for="material-result">Результат</label><textarea id="material-result" bind:value={materialResult}></textarea><label for="limitations">Ограничения</label><textarea id="limitations" bind:value={limitations}></textarea><Button type="submit">Сохранить материал</Button></form>
    </section>
    <div class="my-[18px] flex flex-wrap items-center gap-2"><Button onclick={() => prepareReview(selected!, "topicSuggestion")}>Предложить тему через Codex</Button><Button onclick={() => prepareReview(selected!, "linkSuggestion")}>Предложить дубль или связь</Button></div>
  {/if}
  {#if reviewIdea}<CodexReviewPanel kind={reviewKind} title={reviewCopy[reviewKind].title} packageText={reviewPackageText} response={reviewResponse} error={reviewError} running={reviewRunning} authoredFormulation={formulation} bind:conclusion={reviewConclusion} bind:proposedTopic linkIdeas={library.ideas.filter((idea) => idea.id !== reviewIdea?.id)} bind:linkIdeaId={linkedIdeaId} bind:linkRelation={relation} onStart={startReview} onCancel={() => cancelCodexReview(reviewRequestId)} onConfirmTopic={confirmTopicSuggestion} onConfirmLink={confirmLink} onReject={() => resolveCurrentReview("unchanged")} onRefine={() => resolveCurrentReview("refined", formulation)} onUnchanged={() => resolveCurrentReview("unchanged")} onLater={() => resolveCurrentReview("later")} />{/if}
{/if}

<style>
  h2 { margin: 0 0 8px; font-family: Georgia, serif; font-size: 25px; font-weight: 500; line-height: 1.15; }
  p { margin-top: 0; }
  .stack { display: grid; gap: 14px; }
  .card-actions { display: flex; flex-wrap: wrap; align-items: center; gap: 9px; margin-top: 10px; }
  .idea-workbench { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 16px; margin-top: 20px; }
  .work-card { border: 1px solid var(--color-rule); border-radius: 11px; background: var(--color-paper-raised); padding: 24px; box-shadow: var(--shadow-paper); }
  .idea-workbench form { display: grid; align-content: start; gap: 6px; }
  form label { display: block; margin-bottom: 6px; color: #4d5861; font-size: 12px; font-weight: 700; }
  input, textarea, select { width: 100%; min-width: 0; border: 1px solid #cfd1cd; border-radius: 8px; background: #fffefa; padding: 11px 12px; color: var(--color-ink); outline: none; }
  textarea { min-height: 84px; resize: vertical; } input:focus, textarea:focus, select:focus { border-color: #697c39; box-shadow: 0 0 0 3px var(--color-focus); }
  blockquote { margin: 14px 0; border-left: 3px solid #b8d94a; padding: 6px 0 6px 16px; color: #45515a; line-height: 1.55; }
  details { margin-top: 14px; border-top: 1px solid var(--color-rule); padding-top: 12px; } summary { cursor: pointer; font-weight: 700; }
  .checkbox { display: flex; align-items: center; gap: 8px; margin: 7px 0; }.checkbox input { width: 18px; min-height: 18px; }
  fieldset { margin: 5px 0 10px; border: 1px solid var(--color-rule); border-radius: 8px; padding: 10px 13px; } legend { padding: 0 5px; color: #4d5861; font-size: 12px; font-weight: 700; }
  @media (max-width: 640px) { .idea-workbench { grid-template-columns: 1fr; } }
</style>
