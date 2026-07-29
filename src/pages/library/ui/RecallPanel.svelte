<script lang="ts">
  import { Button, Eyebrow, Surface, TextArea } from "@/shared/ui";
  import type { Idea, LibraryAction, LibraryState } from "@/shared/api";

  let {
    idea,
    library,
    run,
    onReview,
    onClose,
  }: {
    idea: Idea;
    library: LibraryState;
    run: (action: LibraryAction, success?: string) => Promise<void>;
    onReview: (answer: string) => void;
    onClose: () => void;
  } = $props();

  let answer = $state("");
  let revealed = $state(false);
  let nextDate = $state("");

  async function finish(rating: string, message: string) {
    const nextAt = nextDate ? Math.floor(new Date(`${nextDate}T12:00:00`).getTime() / 1000) : null;
    await run({ kind: "completeRecall", ideaId: idea.id, answer, rating, nextAt }, message);
    onClose();
  }
</script>

<Surface class="max-w-[760px]" ariaLabel="Восстановление знания">
  <Eyebrow>Восстановление знания · источник скрыт до ответа</Eyebrow>
  <h2 class="mb-2 font-serif text-2xl font-medium leading-tight">В какой ситуации полезна эта идея?</h2>
  <p>Назовите суть, условия применения и ограничения своими словами.</p>
  <TextArea id="recall-answer" label="Мой ответ" bind:value={answer} disabled={revealed} />
  {#if !revealed}
    <Button variant="primary" onclick={() => revealed = answer.trim().length > 0}>Свериться с идеей</Button>
  {:else}
    <div class="my-[18px] rounded-lg bg-leaf-soft p-[18px]">
      <Eyebrow>Исходная идея</Eyebrow>
      <h2 class="mb-2 font-serif text-2xl font-medium leading-tight">{idea.formulation}</h2>
      {#each library.experiments.filter((item) => item.ideaId === idea.id) as experiment}
        <p><b>Результат применения:</b> {experiment.result}. {experiment.conclusion}</p>
      {/each}
    </div>
    <div class="my-3 flex flex-wrap gap-2"><Button onclick={() => onReview(answer)}>Попросить Codex указать пробелы</Button></div>
    <label class="mb-1.5 block text-xs font-bold text-slate-600" for="recall-next">Перенести предложенное восстановление (необязательно)</label>
    <input class="w-full rounded-lg border border-slate-300 bg-white px-3 py-2.5 focus:border-leaf focus:outline-none focus:ring-3 focus:ring-leaf/20" id="recall-next" type="date" bind:value={nextDate} />
    <p class="mt-4"><b>Самооценка остаётся за вами:</b></p>
    <div class="flex flex-wrap gap-2">
      <Button onclick={() => finish("confident", "Следующее восстановление предложено через 30 дней")}>Уверенно</Button>
      <Button onclick={() => finish("partial", "Следующее восстановление предложено через 7 дней")}>Частично</Button>
      <Button onclick={() => finish("missed", "Следующее восстановление предложено завтра")}>Не восстановил</Button>
    </div>
  {/if}
  <div class="mt-4"><Button onclick={onClose}>Закрыть</Button></div>
</Surface>
