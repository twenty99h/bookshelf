<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { Button, Eyebrow, Surface, TextArea } from "@/shared/ui";
  import { cancelCodexReview, commandErrorMessage, runCodexReview, type Idea, type LibraryAction, type LibraryState } from "@/shared/api";

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
  let recallAnswer = $state("");
  let recallRevealed = $state(false);
  let materialTitle = $state("");
  let problem = $state("");
  let materialIdea = $state("");
  let example = $state("");
  let materialResult = $state("");
  let limitations = $state("");
  let recallNextDate = $state("");
  let reviewIdea = $state<Idea | null>(null);
  let reviewKind = $state("ideaReview");
  let reviewPackageText = $state("");
  let reviewRequestId = $state("");
  let reviewResponse = $state("");
  let reviewError = $state("");
  let reviewRunning = $state(false);
  let reviewConclusion = $state("");
  let proposedTopic = $state("");
  let eventUnlisten: UnlistenFn | undefined;

  let selected = $derived(library.ideas.find((idea) => idea.id === selectedId));
  function selectIdea(idea: Idea) { selectedId = idea.id; formulation = idea.formulation; assignments = [...idea.assignments]; }
  async function saveIdea(event: SubmitEvent) { event.preventDefault(); if (!selected) return; await run({ kind: "updateIdea", ideaId: selected.id, formulation, assignments }, "Новая версия формулировки сохранена"); }
  async function completeExperiment(event: SubmitEvent) { event.preventDefault(); if (!selected) return; await run({ kind: "completeExperiment", ideaId: selected.id, situation, action: actionTaken, result: observedResult, conclusion, successful }, "Практический эксперимент завершён"); }
  async function saveMaterial(event: SubmitEvent) { event.preventDefault(); if (!selected) return; await run({ kind: "saveMaterial", title: materialTitle, problem, idea: materialIdea, example, result: materialResult, limitations, ideaIds: [selected.id] }, "Материал для передачи сохранён"); }
  function buildReviewPackage(idea: Idea, kind: string) {
    const fragment = idea.fragments[0];
    const question = kind === "recallGaps" ? "Какие существенные пробелы есть в моём ответе без выставления самооценки?" : kind === "topicSuggestion" ? "Предложи одну подходящую тему знаний и объясни связь." : kind === "linkSuggestion" ? "Есть ли среди перечисленных идей возможный дубль или смысловая связь?" : "При каких условиях моя формулировка неточна или неприменима?";
    const related = kind === "linkSuggestion" ? `Кандидаты для сравнения: ${library.ideas.filter((candidate) => candidate.id !== idea.id).map((candidate) => candidate.formulation).join("; ") || "нет"}` : "";
    return [
      "Инструкция: укажи возможные пробелы и ограничения; не переписывай идею за читателя и не выставляй итоговую оценку.",
      `Источник: ${bookTitle(idea.bookId)}, ${idea.section}${fragment ? `, стр. ${fragment.page}` : ""}`,
      fragment ? `Выбранный фрагмент: ${fragment.excerpt}` : "",
      `Авторская формулировка: ${idea.formulation}`,
      kind === "recallGaps" ? `Ответ читателя: ${recallAnswer}` : "",
      related,
      `Вопрос: ${question}`,
      "Критерии ответа: точность, существенные ограничения, связь с показанным источником; никаких автоматических изменений.",
    ].filter(Boolean).join("\n\n");
  }
  function prepareReview(idea: Idea, kind = "ideaReview") {
    if (kind === "ideaReview" && selectedId !== idea.id) selectIdea(idea);
    reviewIdea = idea;
    reviewKind = kind;
    reviewPackageText = buildReviewPackage(idea, kind);
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
  async function finishRecall(rating: string, message: string) {
    if (!recallIdea) return;
    const nextAt = recallNextDate ? Math.floor(new Date(`${recallNextDate}T12:00:00`).getTime() / 1000) : null;
    await run({ kind: "completeRecall", ideaId: recallIdea.id, answer: recallAnswer, rating, nextAt }, message);
    recallIdea = null; recallAnswer = ""; recallRevealed = false; recallNextDate = ""; reviewIdea = null;
  }
  async function resolveCurrentReview(decision: "refined" | "unchanged" | "later", authoredFormulation = "") {
    if (!reviewIdea) return;
    await run({ kind: "resolveReview", ideaId: reviewIdea.id, decision, formulation: authoredFormulation, conclusion: reviewConclusion }, decision === "later" ? "Проверка оставлена в долге изучения" : "Решение по проверке сохранено; полный ответ удалён");
    if (decision !== "later") { reviewIdea = null; reviewResponse = ""; reviewPackageText = ""; }
  }
  async function confirmTopicSuggestion() {
    if (!reviewIdea || !proposedTopic.trim()) return;
    await run({ kind: "createTopic", name: proposedTopic }, "Предложенная тема подтверждена");
    await resolveCurrentReview("unchanged");
    proposedTopic = "";
  }
  async function confirmLink() {
    if (!selected || !linkedIdeaId) return;
    await run({ kind: "linkIdeas", fromIdeaId: selected.id, toIdeaId: linkedIdeaId, relation }, "Связь идей подтверждена");
    if (reviewKind === "linkSuggestion" && reviewIdea?.id === selected.id) await resolveCurrentReview("unchanged");
  }
  onMount(() => {
    void listen<{ requestId: string; kind: string; text: string }>("codex-review-event", (event) => {
      if (event.payload.requestId === reviewRequestId && event.payload.kind === "delta") reviewResponse += event.payload.text;
    }).then((stop) => { eventUnlisten = stop; });
    return () => eventUnlisten?.();
  });
</script>

{#if recallIdea}
  <Surface class="max-w-[760px]" ariaLabel="Восстановление знания">
    <Eyebrow>Восстановление знания · источник скрыт до ответа</Eyebrow>
    <h2>В какой ситуации полезна эта идея?</h2>
    <p>Назовите суть, условия применения и ограничения своими словами.</p>
    <TextArea id="recall-answer" label="Мой ответ" bind:value={recallAnswer} disabled={recallRevealed} />
    {#if !recallRevealed}<Button variant="primary" onclick={() => recallRevealed = recallAnswer.trim().length > 0}>Свериться с идеей</Button>{:else}
      <div class="reveal"><Eyebrow>Исходная идея</Eyebrow><h2>{recallIdea.formulation}</h2>{#each library.experiments.filter((item) => item.ideaId === recallIdea?.id) as experiment}<p><b>Результат применения:</b> {experiment.result}. {experiment.conclusion}</p>{/each}</div>
      <div class="review-actions"><Button onclick={() => prepareReview(recallIdea!, "recallGaps")}>Попросить Codex указать пробелы</Button></div>
      <label for="recall-next">Перенести предложенное восстановление (необязательно)</label><input id="recall-next" type="date" bind:value={recallNextDate} />
      <p><b>Самооценка остаётся за вами:</b></p><div class="card-actions"><Button onclick={() => finishRecall("confident", "Следующее восстановление предложено через 30 дней")}>Уверенно</Button><Button onclick={() => finishRecall("partial", "Следующее восстановление предложено через 7 дней")}>Частично</Button><Button onclick={() => finishRecall("missed", "Следующее восстановление предложено завтра")}>Не восстановил</Button></div>
    {/if}
    {#if reviewIdea?.id === recallIdea.id}{@render ReviewPanel()}{/if}
    <Button onclick={() => { recallIdea = null; recallAnswer = ""; recallRevealed = false; }}>Закрыть</Button>
  </Surface>
{:else}
  <section class="stack">
    {#each library.ideas as idea}
      <Surface><Eyebrow>{bookTitle(idea.bookId)} · {idea.section}</Eyebrow><h2>{idea.formulation}</h2><p>Назначения: {idea.assignments.join(", ")}</p><div class="card-actions"><Button onclick={() => selectIdea(idea)}>Развить идею</Button><Button onclick={() => { recallIdea = idea; recallRevealed = false; }}>Восстановить знание</Button><Button onclick={() => prepareReview(idea)}>Проверить идею</Button></div><details><summary>Источник и история</summary>{#each idea.fragments as fragment}<blockquote>стр. {fragment.page}: {fragment.excerpt}</blockquote>{/each}{#each idea.versions as version}<p>{new Date(version.savedAt * 1000).toLocaleString("ru")}: {version.formulation}</p>{/each}</details></Surface>
    {/each}
  </section>

  {#if selected}
    <section class="idea-workbench">
      <form class="work-card" onsubmit={saveIdea}><p class="eyebrow">Авторская формулировка</p><h2>Развить идею</h2><label for="formulation">Текущая формулировка</label><textarea id="formulation" bind:value={formulation}></textarea><fieldset><legend>Назначения идеи</legend><label class="checkbox"><input type="checkbox" value="recall" bind:group={assignments} /> Восстановление</label><label class="checkbox"><input type="checkbox" value="transfer" bind:group={assignments} /> Передача</label><label class="checkbox"><input type="checkbox" value="experiment" bind:group={assignments} /> Практический эксперимент</label><label class="checkbox"><input type="checkbox" value="mastered" bind:group={assignments} /> Уже освоено</label></fieldset><Button type="submit">Сохранить новую версию</Button></form>

      <form class="work-card" onsubmit={(event) => { event.preventDefault(); if (topicId) run({ kind: "assignTopic", ideaId: selected!.id, topicId }, "Идея добавлена в тему"); }}><p class="eyebrow">Организация знаний</p><h2>Тема и связь</h2><label for="topic">Тема</label><select id="topic" bind:value={topicId}><option value="">Выберите тему</option>{#each library.topics as topic}<option value={topic.id}>{topic.name}</option>{/each}</select><Button type="submit">Добавить в тему</Button><label for="linked">Связанная идея</label><select id="linked" bind:value={linkedIdeaId}><option value="">Выберите идею</option>{#each library.ideas.filter((idea) => idea.id !== selected?.id) as idea}<option value={idea.id}>{idea.formulation}</option>{/each}</select><select aria-label="Тип связи" bind:value={relation}><option value="complements">Дополняет</option><option value="clarifies">Уточняет</option><option value="contradicts">Противоречит</option></select><Button onclick={confirmLink}>Подтвердить связь</Button></form>

      <form class="work-card" onsubmit={completeExperiment}><p class="eyebrow">Практический эксперимент</p><h2>Зафиксировать результат</h2><label for="situation">Ситуация</label><textarea id="situation" bind:value={situation}></textarea><label for="action-taken">Действие</label><textarea id="action-taken" bind:value={actionTaken}></textarea><label for="observed">Наблюдаемый результат</label><textarea id="observed" bind:value={observedResult}></textarea><label for="conclusion">Мой вывод</label><textarea id="conclusion" bind:value={conclusion}></textarea><label class="checkbox"><input type="checkbox" bind:checked={successful} /> Результат оказался положительным</label><Button type="submit">Завершить эксперимент</Button></form>

      <form class="work-card" onsubmit={saveMaterial}><p class="eyebrow">Передача знания</p><h2>Авторский Markdown-материал</h2><label for="material-title">Название</label><input id="material-title" bind:value={materialTitle} /><label for="problem">Проблема</label><textarea id="problem" bind:value={problem}></textarea><label for="material-idea">Идея</label><textarea id="material-idea" bind:value={materialIdea}></textarea><label for="example">Пример применения</label><textarea id="example" bind:value={example}></textarea><label for="material-result">Результат</label><textarea id="material-result" bind:value={materialResult}></textarea><label for="limitations">Ограничения</label><textarea id="limitations" bind:value={limitations}></textarea><Button type="submit">Сохранить материал</Button></form>
    </section>
    <div class="card-actions suggestion-actions"><Button onclick={() => prepareReview(selected!, "topicSuggestion")}>Предложить тему через Codex</Button><Button onclick={() => prepareReview(selected!, "linkSuggestion")}>Предложить дубль или связь</Button></div>
  {/if}
  {#if reviewIdea}{@render ReviewPanel()}{/if}
{/if}

{#snippet ReviewPanel()}
  <Surface class="mt-5" ariaLabel="Проверка через Codex">
    <Eyebrow>Передача только после подтверждения</Eyebrow>
    <h2>{reviewKind === "recallGaps" ? "Проверка пробелов ответа" : reviewKind === "topicSuggestion" ? "Предложение темы" : reviewKind === "linkSuggestion" ? "Предложение связи" : "Проверка идеи"}</h2>
    <p>Codex получит ровно этот пакет. Полный PDF, эксперименты и другие заметки не добавляются.</p>
    <pre class="review-package">{reviewPackageText}</pre>
    <div class="card-actions"><Button variant="primary" disabled={reviewRunning} onclick={startReview}>{reviewRunning ? "Проверка идёт…" : "Подтвердить и отправить"}</Button><Button onclick={() => navigator.clipboard.writeText(reviewPackageText)}>Скопировать для внешнего чата</Button>{#if reviewRunning}<Button onclick={() => cancelCodexReview(reviewRequestId)}>Отменить</Button>{/if}</div>
    {#if reviewResponse}<div class="codex-response" aria-live="polite"><Eyebrow>Ответ Codex — временный</Eyebrow><p>{reviewResponse}</p></div>{/if}
    {#if reviewError}<p role="alert">{reviewError}. Пакет можно скопировать во внешний чат; остальные функции доступны.</p>{/if}
    {#if reviewResponse && !reviewRunning && reviewIdea}
      {#if reviewKind === "topicSuggestion"}
        <label for="proposed-topic">Подтверждаемое название темы</label><input id="proposed-topic" bind:value={proposedTopic} /><div class="card-actions"><Button disabled={!proposedTopic.trim()} onclick={confirmTopicSuggestion}>Подтвердить тему</Button><Button onclick={() => resolveCurrentReview("unchanged")}>Отклонить</Button></div>
      {:else if reviewKind === "linkSuggestion"}
        <p>Предложение само ничего не меняет. Выберите идею и тип связи в форме выше, затем нажмите «Подтвердить связь».</p><Button onclick={() => resolveCurrentReview("unchanged")}>Отклонить</Button>
      {:else}
        <label for="review-conclusion">Мой вывод (необязательно)</label><textarea id="review-conclusion" bind:value={reviewConclusion}></textarea><div class="card-actions"><Button onclick={() => resolveCurrentReview("refined", formulation)}>Уточнить своей формулировкой</Button><Button onclick={() => resolveCurrentReview("unchanged")}>Оставить без изменений</Button><Button onclick={() => resolveCurrentReview("later")}>Разобрать позже</Button></div>
      {/if}
    {/if}
  </Surface>
{/snippet}

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
  .reveal { margin: 18px 0; border-radius: 8px; background: #eaf1c8; padding: 18px; }
  .review-package { max-height: 260px; overflow: auto; border: 1px solid var(--color-rule); border-radius: 8px; background: #f4f4ef; padding: 14px; white-space: pre-wrap; font: 12px/1.55 ui-monospace, monospace; }
  .codex-response { margin: 16px 0; border-left: 4px solid #72843d; background: var(--color-leaf-soft); padding: 16px; white-space: pre-wrap; }
  .suggestion-actions { margin: 18px 0; }
  @media (max-width: 640px) { .idea-workbench { grid-template-columns: 1fr; } }
</style>
