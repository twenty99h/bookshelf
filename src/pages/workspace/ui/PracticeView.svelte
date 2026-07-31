<script lang="ts">
  import { onMount } from "svelte";
  import { Check } from "@lucide/svelte";
  import { Button, SelectField, TextArea, TextField } from "@/shared/ui";
  import type { ExperimentStatus, LibraryState, RecallRating } from "@/shared/api";

  let {
    library,
    onCompleteRecall,
    onRescheduleRecall,
    onStartRecallNow,
    onCreateExperiment,
    onAdvanceExperiment,
  }: {
    library: LibraryState;
    onCompleteRecall: (answer: string, rating: RecallRating) => Promise<void>;
    onRescheduleRecall: (days: number) => Promise<void>;
    onStartRecallNow: () => Promise<void>;
    onCreateExperiment: (draft: {
      ideaId: string;
      situation: string;
      action: string;
      nextStep: string;
    }) => Promise<void>;
    onAdvanceExperiment: (
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
    ) => Promise<void>;
  } = $props();

  let recallAnswer = $state("");
  let recallRevealed = $state(false);
  let selectedExperimentId = $state("");
  let experimentStep = $state<ExperimentStatus>("intent");
  let experimentSituation = $state("");
  let experimentAction = $state("");
  let experimentResult = $state("");
  let experimentConclusion = $state("");
  let experimentNextStep = $state("");
  let experimentCancellationReason = $state("");
  let newExperimentIdeaId = $state("");
  let newExperimentSituation = $state("");
  let newExperimentAction = $state("");
  let newExperimentNextStep = $state("");

  const recall = $derived(library.recalls.toSorted((a, b) => a.nextAt - b.nextAt)[0] ?? null);
  const recallIdea = $derived(library.ideas.find((idea) => idea.id === recall?.ideaId) ?? null);
  const currentExperiment = $derived(library.experiments.find((item) => item.id === selectedExperimentId) ?? null);
  const experimentIdeas = $derived(library.ideas.filter((idea) => idea.assignments.includes("experiment")));
  const relatedExperiments = $derived(library.experiments.filter((experiment) => experiment.ideaId === recallIdea?.id));

  onMount(() => {
    const firstExperiment = library.experiments[0];
    if (firstExperiment) selectExperiment(firstExperiment.id);
    newExperimentIdeaId = library.ideas.find((idea) => idea.assignments.includes("experiment"))?.id ?? "";
  });

  function selectExperiment(experimentId: string) {
    const experiment = library.experiments.find((item) => item.id === experimentId);
    if (!experiment) return;
    selectedExperimentId = experiment.id;
    experimentStep = experiment.status;
    experimentSituation = experiment.situation;
    experimentAction = experiment.action;
    experimentResult = experiment.result;
    experimentConclusion = experiment.conclusion;
    experimentNextStep = experiment.nextStep;
    experimentCancellationReason = experiment.cancellationReason;
  }

  async function advanceExperiment(status: ExperimentStatus) {
    if (!currentExperiment) return;
    await onAdvanceExperiment(currentExperiment.id, status, {
      situation: experimentSituation,
      action: experimentAction,
      result: experimentResult,
      conclusion: experimentConclusion,
      cancellationReason: experimentCancellationReason,
      nextStep: experimentNextStep,
    });
    experimentStep = status;
  }

  async function createExperiment() {
    await onCreateExperiment({
      ideaId: newExperimentIdeaId,
      situation: newExperimentSituation,
      action: newExperimentAction,
      nextStep: newExperimentNextStep,
    });
    newExperimentSituation = "";
    newExperimentAction = "";
    newExperimentNextStep = "";
  }

  async function startRecallNow() {
    recallRevealed = false;
    recallAnswer = "";
    await onStartRecallNow();
  }

  function bookTitleForIdea(ideaId: string) {
    const idea = library.ideas.find((item) => item.id === ideaId);
    return library.books.find((book) => book.id === idea?.bookId)?.title ?? "Книга не найдена";
  }

  function statusLabel(status: ExperimentStatus) {
    return {
      intent: "Замысел",
      running: "Выполняется",
      reviewing: "Подведение итогов",
      completed: "Завершён",
      cancelled: "Отменён с причиной",
    }[status];
  }

  function recallDate(timestamp: number) {
    return new Intl.DateTimeFormat("ru-RU", { day: "numeric", month: "long", year: "numeric" }).format(
      new Date(timestamp * 1_000),
    );
  }
</script>

<div class="grid gap-6">
  <div class="grid grid-cols-[minmax(0,.85fr)_minmax(420px,1.15fr)] gap-6 max-[1280px]:grid-cols-1">
    <section class="rounded-xl border border-white/8 bg-slate p-7">
      <p class="font-mono text-xs uppercase tracking-[.14em] text-iris">Восстановление знания</p>
      <h2 class="mt-3 text-2xl font-semibold">
        В какой ситуации применима идея из раздела «{recallIdea?.section ?? "книги"}»?
      </h2>
      <p class="mt-3 leading-7 text-mist-dim">
        Опишите подходящую ситуацию, объясните идею своими словами и назовите ограничения.
      </p>
      <div class="mt-6"><TextArea id="recall-answer" label="Мой ответ и ограничения" bind:value={recallAnswer} /></div>
      {#if !recallRevealed}<Button
          variant="primary"
          disabled={!recallAnswer.trim()}
          onclick={() => (recallRevealed = true)}>Свериться с идеей</Button
        >{:else}<div class="mt-5 rounded-lg border border-amber/30 bg-amber/[.06] p-5">
          <small class="font-mono uppercase text-amber">Исходная идея и источник</small>
          <p class="mt-3 leading-7">{recallIdea?.formulation}</p>
          {#if recallIdea?.fragments[0]}<p class="mt-3 text-sm text-mist-dim">
              Страница {recallIdea.fragments[0].page} · {recallIdea.fragments[0].excerpt}
            </p>{/if}
        </div>
        <div class="mt-5">
          <p class="mb-3 text-sm font-semibold">Как удалось восстановить?</p>
          <div class="flex flex-wrap gap-2">
            <Button onclick={() => onCompleteRecall(recallAnswer, "confident")}>Уверенно</Button><Button
              onclick={() => onCompleteRecall(recallAnswer, "partial")}>Частично</Button
            ><Button onclick={() => onCompleteRecall(recallAnswer, "notRecalled")}>Не восстановил</Button>
          </div>
        </div>
        {#if relatedExperiments.length}<div class="mt-5 rounded-lg border border-white/8 p-4">
            <b class="text-sm">Связанные результаты практики</b>
            {#each relatedExperiments as experiment (experiment.id)}<p class="mt-2 text-sm text-mist-dim">
                {experiment.result || "Результат ещё не записан"}
              </p>{/each}
          </div>{/if}{/if}
      {#if recall}<div class="mt-6 border-t border-white/8 pt-5">
          <p class="text-sm text-mist-dim">Следующее восстановление: {recallDate(recall.nextAt)}.</p>
          <div class="mt-3 flex flex-wrap gap-2">
            <Button onclick={() => onRescheduleRecall(7)}>Перенести на 7 дней</Button>
            <Button onclick={startRecallNow}>Начать сейчас</Button>
          </div>
          <p class="mt-3 text-xs text-mist-faint">Без просрочки, календарного сеанса и записи о пропуске.</p>
        </div>{/if}
    </section>

    <section class="rounded-xl border border-white/8 bg-slate p-7">
      <div class="flex items-start justify-between gap-4">
        <div>
          <p class="font-mono text-xs uppercase tracking-[.14em] text-amber">Практический эксперимент</p>
          <h2 class="mt-3 text-2xl font-semibold">{currentExperiment?.situation ?? "Новый замысел"}</h2>
        </div>
        {#if currentExperiment}<span class="rounded-md bg-iris/12 px-3 py-1.5 font-mono text-xs text-iris"
            >{statusLabel(currentExperiment.status)}</span
          >{/if}
      </div>
      {#if currentExperiment}<p class="mt-4 text-sm text-mist-dim">
          {bookTitleForIdea(currentExperiment.ideaId)} · эксперимент остаётся видимым независимо от статуса книги
        </p>
        <div class="mt-6 grid grid-cols-[22px_1fr] gap-x-4 gap-y-5">
          {@render step("Замысел", true, "Ситуация и проверяемое действие сохранены.")}
          {@render step("Выполняется", experimentStep !== "intent", "Реальное действие без обязательного дедлайна.")}
          {@render step(
            "Подведение итогов",
            ["reviewing", "completed"].includes(experimentStep),
            "Наблюдаемый результат и вывод.",
          )}
          {@render step(
            "Завершён",
            experimentStep === "completed",
            "Отрицательный результат тоже считается применением.",
          )}
        </div>
        <div class="mt-7 grid gap-4">
          <TextArea id="experiment-situation" label="Ситуация" bind:value={experimentSituation} required />
          <TextArea id="experiment-action" label="Действие" bind:value={experimentAction} required />
          <TextArea id="experiment-result" label="Наблюдаемый результат" bind:value={experimentResult} />
          <TextArea id="experiment-conclusion" label="Авторский вывод" bind:value={experimentConclusion} />
          <TextField id="experiment-next" label="Следующий шаг (без даты)" bind:value={experimentNextStep} />
        </div>
        <div class="mt-5 flex flex-wrap gap-2">
          {#if !["completed", "cancelled"].includes(experimentStep)}<Button
              onclick={() => advanceExperiment(experimentStep)}>Сохранить изменения</Button
            >{/if}
          {#if experimentStep === "intent"}<Button variant="primary" onclick={() => advanceExperiment("running")}
              >Начать выполнение</Button
            >{:else if experimentStep === "running"}<Button
              variant="primary"
              onclick={() => advanceExperiment("reviewing")}>Перейти к итогу</Button
            >{:else if experimentStep === "reviewing"}<Button
              variant="primary"
              disabled={!experimentSituation.trim() ||
                !experimentAction.trim() ||
                !experimentResult.trim() ||
                !experimentConclusion.trim()}
              onclick={() => advanceExperiment("completed")}>Завершить эксперимент</Button
            ><Button onclick={() => advanceExperiment("running")}>Продолжить выполнение</Button>{/if}
        </div>
        {#if !["completed", "cancelled"].includes(experimentStep)}<div class="mt-5 flex items-end gap-3">
            <div class="min-w-0 flex-1">
              <TextField id="experiment-cancel" label="Причина отмены" bind:value={experimentCancellationReason} />
            </div>
            <Button disabled={!experimentCancellationReason.trim()} onclick={() => advanceExperiment("cancelled")}
              >Отменить с причиной</Button
            >
          </div>{/if}
      {/if}
    </section>
  </div>

  <section
    class="grid grid-cols-[minmax(360px,.8fr)_minmax(0,1.2fr)] gap-6 rounded-xl border border-white/8 bg-slate p-7 max-[1280px]:grid-cols-1"
  >
    <div>
      <p class="font-mono text-xs uppercase tracking-[.14em] text-amber">Новый эксперимент</p>
      <h2 class="mt-3 text-xl font-semibold">Сохранить замысел</h2>
      <div class="mt-5 grid gap-4">
        <SelectField
          label="Идея для проверки"
          bind:value={newExperimentIdeaId}
          options={experimentIdeas.map((idea) => ({ value: idea.id, label: idea.formulation }))}
        />
        <TextArea id="new-experiment-situation" label="Ситуация нового замысла" bind:value={newExperimentSituation} />
        <TextArea id="new-experiment-action" label="Проверяемое действие" bind:value={newExperimentAction} />
        <TextField id="new-experiment-next" label="Следующий шаг нового замысла" bind:value={newExperimentNextStep} />
        <Button
          variant="primary"
          disabled={!newExperimentIdeaId || !newExperimentSituation.trim() || !newExperimentAction.trim()}
          onclick={createExperiment}>Создать замысел</Button
        >
      </div>
    </div>
    <div>
      <p class="font-mono text-xs uppercase tracking-[.14em] text-mist-dim">Все эксперименты</p>
      <div class="mt-4 grid gap-2">
        {#each library.experiments as experiment (experiment.id)}<button
            aria-current={experiment.id === selectedExperimentId ? "true" : undefined}
            class="flex items-start gap-4 rounded-lg border border-white/8 bg-night/30 p-4 text-left aria-[current=true]:border-iris/60"
            onclick={() => selectExperiment(experiment.id)}
          >
            <div class="min-w-0">
              <b class="block">{experiment.situation}</b>
              <span class="mt-1 block text-sm text-mist-dim">{bookTitleForIdea(experiment.ideaId)}</span>
              {#if experiment.nextStep}<small class="mt-2 block text-mist-faint">Дальше: {experiment.nextStep}</small
                >{/if}
            </div>
            <span class="ml-auto shrink-0 font-mono text-xs text-iris">{statusLabel(experiment.status)}</span>
          </button>{/each}
      </div>
    </div>
  </section>
</div>

{#snippet step(label: string, active: boolean, detail: string)}<span
    class="mt-1 grid size-[22px] place-items-center rounded-full border {active
      ? 'border-iris-strong bg-iris-strong text-white'
      : 'border-white/15 text-transparent'}"><Check class="size-3" /></span
  >
  <div>
    <b class="text-sm">{label}</b>
    <p class="mt-1 text-sm leading-6 text-mist-dim">{detail}</p>
  </div>
{/snippet}
