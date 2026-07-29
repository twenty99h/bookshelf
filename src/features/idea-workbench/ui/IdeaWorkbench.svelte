<script lang="ts">
  import { Button } from "@/shared/ui";
  import type { Idea, LibraryAction, LibraryState } from "@/shared/api";

  let { library, run, bookTitle }: { library: LibraryState; run: (action: LibraryAction, success?: string) => Promise<void>; bookTitle: (bookId: string) => string } = $props();
  let selectedId = $state("");
  let formulation = $state("");
  let assignment = $state("recall");
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

  let selected = $derived(library.ideas.find((idea) => idea.id === selectedId));
  function selectIdea(idea: Idea) { selectedId = idea.id; formulation = idea.formulation; assignment = idea.assignments[0] ?? "recall"; }
  async function saveIdea(event: SubmitEvent) { event.preventDefault(); if (!selected) return; await run({ kind: "updateIdea", ideaId: selected.id, formulation, assignments: [assignment] }, "Новая версия формулировки сохранена"); }
  async function completeExperiment(event: SubmitEvent) { event.preventDefault(); if (!selected) return; await run({ kind: "completeExperiment", ideaId: selected.id, situation, action: actionTaken, result: observedResult, conclusion, successful }, "Практический эксперимент завершён"); }
  async function saveMaterial(event: SubmitEvent) { event.preventDefault(); if (!selected) return; await run({ kind: "saveMaterial", title: materialTitle, problem, idea: materialIdea, example, result: materialResult, limitations, ideaIds: [selected.id] }, "Материал для передачи сохранён"); }
</script>

{#if recallIdea}
  <section class="recall work-card" aria-label="Восстановление знания">
    <p class="eyebrow">Восстановление знания · источник скрыт до ответа</p>
    <h2>В какой ситуации полезна эта идея?</h2>
    <p>Назовите суть, условия применения и ограничения своими словами.</p>
    <label for="recall-answer">Мой ответ</label><textarea id="recall-answer" bind:value={recallAnswer} disabled={recallRevealed}></textarea>
    {#if !recallRevealed}<Button variant="primary" onclick={() => recallRevealed = recallAnswer.trim().length > 0}>Свериться с идеей</Button>{:else}
      <div class="reveal"><p class="eyebrow">Исходная идея</p><h2>{recallIdea.formulation}</h2>{#each library.experiments.filter((item) => item.ideaId === recallIdea?.id) as experiment}<p><b>Результат применения:</b> {experiment.result}. {experiment.conclusion}</p>{/each}</div>
      <p><b>Самооценка:</b></p><div class="card-actions"><Button onclick={() => run({ kind: "completeRecall", ideaId: recallIdea!.id, answer: recallAnswer, rating: "confident" }, "Следующее восстановление предложено через 30 дней")}>Уверенно</Button><Button onclick={() => run({ kind: "completeRecall", ideaId: recallIdea!.id, answer: recallAnswer, rating: "partial" }, "Следующее восстановление предложено через 7 дней")}>Частично</Button><Button onclick={() => run({ kind: "completeRecall", ideaId: recallIdea!.id, answer: recallAnswer, rating: "missed" }, "Следующее восстановление предложено завтра")}>Не восстановил</Button></div>
    {/if}
    <Button onclick={() => { recallIdea = null; recallAnswer = ""; recallRevealed = false; }}>Закрыть</Button>
  </section>
{:else}
  <section class="stack">
    {#each library.ideas as idea}
      <article class="work-card"><p class="eyebrow">{bookTitle(idea.bookId)} · {idea.section}</p><h2>{idea.formulation}</h2><p>Назначения: {idea.assignments.join(", ")}</p><div class="card-actions"><Button onclick={() => selectIdea(idea)}>Развить идею</Button><Button onclick={() => { recallIdea = idea; recallRevealed = false; }}>Восстановить знание</Button></div><details><summary>Источник и история</summary>{#each idea.fragments as fragment}<blockquote>стр. {fragment.page}: {fragment.excerpt}</blockquote>{/each}{#each idea.versions as version}<p>{new Date(version.savedAt * 1000).toLocaleString("ru")}: {version.formulation}</p>{/each}</details></article>
    {/each}
  </section>

  {#if selected}
    <section class="idea-workbench">
      <form class="work-card" onsubmit={saveIdea}><p class="eyebrow">Авторская формулировка</p><h2>Развить идею</h2><label for="formulation">Текущая формулировка</label><textarea id="formulation" bind:value={formulation}></textarea><label for="assignment">Назначение</label><select id="assignment" bind:value={assignment}><option value="recall">Восстановление</option><option value="transfer">Передача</option><option value="experiment">Практический эксперимент</option><option value="mastered">Уже освоено</option></select><Button type="submit">Сохранить новую версию</Button></form>

      <form class="work-card" onsubmit={(event) => { event.preventDefault(); if (topicId) run({ kind: "assignTopic", ideaId: selected!.id, topicId }, "Идея добавлена в тему"); }}><p class="eyebrow">Организация знаний</p><h2>Тема и связь</h2><label for="topic">Тема</label><select id="topic" bind:value={topicId}><option value="">Выберите тему</option>{#each library.topics as topic}<option value={topic.id}>{topic.name}</option>{/each}</select><Button type="submit">Добавить в тему</Button><label for="linked">Связанная идея</label><select id="linked" bind:value={linkedIdeaId}><option value="">Выберите идею</option>{#each library.ideas.filter((idea) => idea.id !== selected?.id) as idea}<option value={idea.id}>{idea.formulation}</option>{/each}</select><select aria-label="Тип связи" bind:value={relation}><option value="complements">Дополняет</option><option value="clarifies">Уточняет</option><option value="contradicts">Противоречит</option></select><Button onclick={() => linkedIdeaId && run({ kind: "linkIdeas", fromIdeaId: selected!.id, toIdeaId: linkedIdeaId, relation }, "Связь идей подтверждена")}>Подтвердить связь</Button></form>

      <form class="work-card" onsubmit={completeExperiment}><p class="eyebrow">Практический эксперимент</p><h2>Зафиксировать результат</h2><label for="situation">Ситуация</label><textarea id="situation" bind:value={situation}></textarea><label for="action-taken">Действие</label><textarea id="action-taken" bind:value={actionTaken}></textarea><label for="observed">Наблюдаемый результат</label><textarea id="observed" bind:value={observedResult}></textarea><label for="conclusion">Мой вывод</label><textarea id="conclusion" bind:value={conclusion}></textarea><label class="checkbox"><input type="checkbox" bind:checked={successful} /> Результат оказался положительным</label><Button type="submit">Завершить эксперимент</Button></form>

      <form class="work-card" onsubmit={saveMaterial}><p class="eyebrow">Передача знания</p><h2>Авторский Markdown-материал</h2><label for="material-title">Название</label><input id="material-title" bind:value={materialTitle} /><label for="problem">Проблема</label><textarea id="problem" bind:value={problem}></textarea><label for="material-idea">Идея</label><textarea id="material-idea" bind:value={materialIdea}></textarea><label for="example">Пример применения</label><textarea id="example" bind:value={example}></textarea><label for="material-result">Результат</label><textarea id="material-result" bind:value={materialResult}></textarea><label for="limitations">Ограничения</label><textarea id="limitations" bind:value={limitations}></textarea><Button type="submit">Сохранить материал</Button></form>
    </section>
  {/if}
{/if}
