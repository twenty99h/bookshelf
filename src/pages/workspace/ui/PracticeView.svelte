<script lang="ts">
  import { Check } from "@lucide/svelte";
  import { Button, TextArea, TextField } from "@/shared/ui";
  import type { LibraryState, RecallRating } from "@/shared/api";
  let {
    library,
    recallAnswer = $bindable(),
    recallRevealed = $bindable(),
    experimentStep,
    experimentNextStep = $bindable(),
    experimentCancellationReason = $bindable(),
    onCompleteRecall,
    onAdvanceExperiment,
  }: {
    library: LibraryState;
    recallAnswer: string;
    recallRevealed: boolean;
    experimentStep: string;
    experimentNextStep: string;
    experimentCancellationReason: string;
    onCompleteRecall: (rating: RecallRating) => Promise<void>;
    onAdvanceExperiment: (status: "reviewing" | "completed" | "cancelled") => Promise<void>;
  } = $props();
</script>

<div class="grid grid-cols-[minmax(0,.85fr)_minmax(420px,1.15fr)] gap-6">
  <section class="rounded-xl border border-white/8 bg-slate p-7">
    <p class="font-mono text-xs uppercase tracking-[.14em] text-iris">Восстановление знания</p>
    <h2 class="mt-3 text-2xl font-semibold">Что меняется при отказе лидера?</h2>
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
        <p class="mt-3 leading-7">{library!.ideas[0]?.formulation}</p>
      </div>
      <div class="mt-5">
        <p class="mb-3 text-sm font-semibold">Как удалось восстановить?</p>
        <div class="flex gap-2">
          <Button onclick={() => onCompleteRecall("confident")}>Уверенно</Button><Button
            onclick={() => onCompleteRecall("partial")}>Частично</Button
          ><Button onclick={() => onCompleteRecall("notRecalled")}>Не восстановил</Button>
        </div>
        <p class="mt-4 text-sm text-mist-dim">
          Следующее восстановление предложено на 3 августа. Его можно перенести или запустить раньше.
        </p>
      </div>{/if}
  </section>
  <section class="rounded-xl border border-white/8 bg-slate p-7">
    <div class="flex items-start justify-between">
      <div>
        <p class="font-mono text-xs uppercase tracking-[.14em] text-amber">Практический эксперимент</p>
        <h2 class="mt-3 text-2xl font-semibold">Явная смена владельца журнала</h2>
      </div>
      <span class="rounded-md bg-iris/12 px-3 py-1.5 font-mono text-xs text-iris"
        >{experimentStep === "running" ? "Выполняется" : "Подведение итогов"}</span
      >
    </div>
    <p class="mt-5 text-sm text-mist-dim">Designing Data-Intensive Applications · идея о риске единого лидера</p>
    <div class="mt-7 grid grid-cols-[22px_1fr] gap-x-4 gap-y-6">
      {@render step("Замысел", true, "Проверить, делает ли явная аренда отказ понятнее команде.")}{@render step(
        "Выполняется",
        true,
        "Применяем переход состояния в журнале конфигурации.",
      )}{@render step(
        "Подведение итогов",
        experimentStep === "review",
        "Зафиксировать наблюдаемый результат и авторский вывод.",
      )}{@render step("Завершён", false, "Положительный результат не обязателен.")}
    </div>
    <div class="mt-8 flex gap-2">
      <Button variant="primary" onclick={() => onAdvanceExperiment("reviewing")}>Перейти к итогу</Button><Button
        onclick={() => onAdvanceExperiment("completed")}>Завершить с результатом</Button
      >
    </div>
    <div class="mt-5 grid grid-cols-2 gap-4">
      <TextField id="experiment-next" label="Следующий шаг (без даты)" bind:value={experimentNextStep} />
      <TextField id="experiment-cancel" label="Причина отмены" bind:value={experimentCancellationReason} />
    </div>
    <Button disabled={!experimentCancellationReason.trim()} onclick={() => onAdvanceExperiment("cancelled")}
      >Отменить с причиной</Button
    >
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
