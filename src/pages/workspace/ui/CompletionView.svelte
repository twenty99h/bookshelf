<script lang="ts">
  import { Button, CheckboxField, SelectField, TextArea } from "@/shared/ui";
  import type { Book, CompletionWorkDecision, LibraryState } from "@/shared/api";

  type WorkItem = { id: string; kind: CompletionWorkDecision["kind"]; label: string };

  let {
    step,
    library,
    selectedBook,
    significantIdeas = $bindable(),
    retrospective = $bindable(),
    unfinishedWorkDecision = $bindable(),
    continuingWork = $bindable(),
    workItems,
    workDecisions,
    onCompleteReading,
    onSaveStep,
    onSetWorkDecision,
    hasDecisions,
    onFinish,
  }: {
    step: number;
    library: LibraryState;
    selectedBook: Book | null;
    significantIdeas: string[];
    retrospective: string;
    unfinishedWorkDecision: string;
    continuingWork: string;
    workItems: WorkItem[];
    workDecisions: CompletionWorkDecision[];
    onCompleteReading: () => Promise<void>;
    onSaveStep: (step: number) => Promise<void>;
    onSetWorkDecision: (workId: string, kind: CompletionWorkDecision["kind"], decision: string) => void;
    hasDecisions: (kind: "experiment" | "other") => boolean;
    onFinish: () => Promise<void>;
  } = $props();

  function toggleIdea(ideaId: string, checked: boolean) {
    significantIdeas = checked ? [...significantIdeas, ideaId] : significantIdeas.filter((id) => id !== ideaId);
  }
</script>

<div class="mx-auto max-w-5xl">
  <div class="mb-8 flex items-center justify-between">
    {#each ["Чтение", "Идеи", "Ретроспектива", "Работа", "Эксперименты", "Подтверждение"] as label, index (label)}<div
        class="flex items-center {index < 5 ? 'flex-1' : ''}"
      >
        <span
          class="grid size-8 place-items-center rounded-full border font-mono text-xs {step >= index + 1
            ? 'border-iris-strong bg-iris-strong text-white'
            : 'border-white/12 text-mist-dim'}">{index + 1}</span
        >{#if index < 5}<span class="mx-2 h-px flex-1 bg-white/10"></span>{/if}
      </div>{/each}
  </div>
  <section class="rounded-xl border border-white/8 bg-slate p-8">
    <p class="font-mono text-xs uppercase tracking-[.14em] text-iris">Шаг {step} из 6</p>
    {#if step === 1}
      <h2 class="mt-3 text-3xl font-semibold">Чтение действительно завершено?</h2>
      <p class="mt-4 max-w-2xl leading-7 text-mist-dim">
        Это фиксирует окончание работы с текстом, но ещё не завершает изучение книги.
      </p>
      <div class="mt-7"><Button variant="primary" onclick={onCompleteReading}>Чтение завершено</Button></div>
    {:else if step === 2}
      <h2 class="mt-3 text-3xl font-semibold">Выберите 3–7 значимых идей</h2>
      <div class="mt-6 grid gap-3">
        {#each library.ideas.filter((idea) => idea.bookId === selectedBook?.id) as idea (idea.id)}<CheckboxField
            id={`significant-${idea.id}`}
            label={idea.formulation}
            checked={significantIdeas.includes(idea.id)}
            onCheckedChange={(checked) => toggleIdea(idea.id, checked)}
          />{/each}
      </div>
      <div class="mt-7">
        <Button
          variant="primary"
          disabled={significantIdeas.length < 3 || significantIdeas.length > 7}
          onclick={() => onSaveStep(3)}>Продолжить</Button
        >
      </div>
    {:else if step === 3}
      <h2 class="mt-3 text-3xl font-semibold">Ретроспектива книги</h2>
      <p class="mt-3 text-mist-dim">Что изменилось в вашем понимании или действиях? Итог пишете вы.</p>
      <div class="mt-6"><TextArea id="retrospective" label="Авторский итог" bind:value={retrospective} /></div>
      <Button variant="primary" disabled={!retrospective.trim()} onclick={() => onSaveStep(4)}
        >Сохранить черновик и продолжить</Button
      >
    {:else if step === 4}
      <h2 class="mt-3 text-3xl font-semibold">Решите судьбу незавершённой работы</h2>
      <p class="mt-4 leading-7 text-mist-dim">
        Каждому открытому черновику, проверке и восстановлению нужно отдельное читательское решение.
      </p>
      <div class="mt-6 grid gap-4">
        {#each workItems.filter((item) => item.kind !== "experiment") as item (`${item.kind}-${item.id}`)}<div
            class="grid grid-cols-[minmax(0,1fr)_280px] items-center gap-4 rounded-lg border border-white/8 bg-night/30 p-4"
          >
            <span>{item.label}</span><SelectField
              label="Решение"
              value={workDecisions.find((entry) => entry.workId === item.id && entry.kind === item.kind)?.decision ??
                ""}
              options={[
                { value: "finish-later", label: "Разобрать позже" },
                { value: "keep-reference", label: "Оставить как справку" },
                { value: "close-separately", label: "Закрыть отдельно" },
              ]}
              onValueChange={(value) => onSetWorkDecision(item.id, item.kind, value)}
            />
          </div>{:else}<p class="text-sm text-mist-dim">Открытых черновиков, проверок и восстановлений нет.</p>{/each}
      </div>
      <div class="mt-6">
        <TextArea
          id="unfinished-work-decision"
          label="Общий комментарий к решениям"
          bind:value={unfinishedWorkDecision}
        />
      </div>
      <Button
        variant="primary"
        disabled={!hasDecisions("other") || !unfinishedWorkDecision.trim()}
        onclick={() => onSaveStep(5)}>Продолжить</Button
      >
    {:else if step === 5}
      <h2 class="mt-3 text-3xl font-semibold">Продолжающиеся эксперименты</h2>
      <p class="mt-4 leading-7 text-mist-dim">
        Эксперименты не завершаются автоматически вместе с изучением книги и сохраняют текущее состояние.
      </p>
      <div class="mt-6 grid gap-4">
        {#each workItems.filter((item) => item.kind === "experiment") as item (item.id)}<div
            class="grid grid-cols-[minmax(0,1fr)_280px] items-center gap-4 rounded-lg border border-white/8 bg-night/30 p-4"
          >
            <span>{item.label}</span><SelectField
              label="Решение"
              value={workDecisions.find((entry) => entry.workId === item.id && entry.kind === item.kind)?.decision ??
                ""}
              options={[
                { value: "continue", label: "Продолжить после итога" },
                { value: "pause", label: "Приостановить вручную" },
                { value: "review", label: "Подвести итог отдельно" },
              ]}
              onValueChange={(value) => onSetWorkDecision(item.id, item.kind, value)}
            />
          </div>{:else}<p class="text-sm text-mist-dim">Продолжающихся экспериментов нет.</p>{/each}
      </div>
      <div class="mt-6">
        <TextArea id="continuing-work" label="Комментарий к продолжающейся практике" bind:value={continuingWork} />
      </div>
      <Button variant="primary" disabled={!hasDecisions("experiment")} onclick={() => onSaveStep(6)}
        >Перейти к подтверждению</Button
      >
    {:else}
      <h2 class="mt-3 text-3xl font-semibold">Подтвердите итог изучения</h2>
      <div class="mt-6 grid gap-3 rounded-lg border border-white/8 bg-night/30 p-5 text-sm">
        <p><b>Значимых идей:</b> {significantIdeas.length}</p>
        <p><b>Решений по открытой работе:</b> {workDecisions.length}</p>
        <p><b>Ретроспектива:</b> {retrospective}</p>
      </div>
      <p class="mt-5 text-sm leading-6 text-mist-dim">
        Завершится только цикл изучения. Продолжающиеся эксперименты останутся активными.
      </p>
      <Button variant="primary" onclick={onFinish}>Завершить изучение</Button>
    {/if}
  </section>
</div>
