<script lang="ts">
  import type { Idea, ReviewKind } from "@/shared/api";
  import { Button, Eyebrow, SelectField, Surface, TextArea, TextField } from "@/shared/ui";

  let {
    kind,
    title,
    packageText,
    response,
    error,
    running,
    authoredFormulation,
    conclusion = $bindable(""),
    proposedTopic = $bindable(""),
    linkIdeas = [],
    linkIdeaId = $bindable(""),
    linkRelation = $bindable("complements"),
    onStart,
    onCancel,
    onConfirmTopic,
    onConfirmLink,
    onReject,
    onRefine,
    onUnchanged,
    onLater,
  }: {
    kind: ReviewKind;
    title: string;
    packageText: string;
    response: string;
    error: string;
    running: boolean;
    authoredFormulation: string;
    conclusion?: string;
    proposedTopic?: string;
    linkIdeas?: Idea[];
    linkIdeaId?: string;
    linkRelation?: string;
    onStart: () => void;
    onCancel: () => void;
    onConfirmTopic: () => void;
    onReject: () => void;
    onConfirmLink: () => void;
    onRefine: () => void;
    onUnchanged: () => void;
    onLater: () => void;
  } = $props();
</script>

<Surface class="mt-5" ariaLabel="Проверка через Codex">
  <Eyebrow>Передача только после подтверждения</Eyebrow>
  <h2 class="mb-2 font-display text-[25px] font-medium leading-tight">{title}</h2>
  <p>Codex получит ровно этот пакет. Полный PDF, эксперименты и другие заметки не добавляются.</p>
  <pre
    class="max-h-[260px] overflow-auto whitespace-pre-wrap rounded-lg border border-rule bg-[#f4f4ef] p-3.5 font-mono text-xs leading-relaxed">{packageText}</pre>
  <div class="mt-2.5 flex flex-wrap items-center gap-2">
    <Button variant="primary" disabled={running} onclick={onStart}
      >{running ? "Проверка идёт…" : "Подтвердить и отправить"}</Button
    ><Button onclick={() => navigator.clipboard.writeText(packageText)}>Скопировать для внешнего чата</Button
    >{#if running}<Button onclick={onCancel}>Отменить</Button>{/if}
  </div>
  {#if response}<div class="my-4 whitespace-pre-wrap border-l-4 border-[#72843d] bg-leaf-soft p-4" aria-live="polite">
      <Eyebrow>Ответ Codex — временный</Eyebrow>
      <p>{response}</p>
    </div>{/if}
  {#if error}<p role="alert">{error}. Пакет можно скопировать во внешний чат; остальные функции доступны.</p>{/if}
  {#if response && !running}
    {#if kind === "topicSuggestion"}
      <TextField id="proposed-topic" label="Подтверждаемое название темы" bind:value={proposedTopic} />
      <div class="mt-2.5 flex flex-wrap gap-2">
        <Button disabled={!proposedTopic.trim()} onclick={onConfirmTopic}>Подтвердить тему</Button><Button
          onclick={onReject}>Отклонить</Button
        >
      </div>
    {:else if kind === "linkSuggestion"}
      <p>Ответ выше — один кандидат. Он ничего не меняет, пока вы не выберете точную идею и связь.</p>
      <SelectField
        label="Связанная идея"
        bind:value={linkIdeaId}
        placeholder="Выберите идею из кандидата"
        options={linkIdeas.map((idea) => ({ value: idea.id, label: idea.formulation }))}
      />
      <div class="mt-3">
        <SelectField
          label="Тип связи"
          bind:value={linkRelation}
          options={[
            { value: "complements", label: "Дополняет" },
            { value: "clarifies", label: "Уточняет" },
            { value: "contradicts", label: "Противоречит" },
          ]}
        />
      </div>
      <div class="mt-2.5 flex flex-wrap gap-2">
        <Button disabled={!linkIdeaId} onclick={onConfirmLink}>Подтвердить этого кандидата</Button><Button
          onclick={onReject}>Отклонить</Button
        >
      </div>
    {:else}
      <TextArea id="review-conclusion" label="Мой вывод (необязательно)" bind:value={conclusion} />
      <div class="mt-2.5 flex flex-wrap gap-2">
        <Button disabled={!authoredFormulation.trim()} onclick={onRefine}>Уточнить своей формулировкой</Button><Button
          onclick={onUnchanged}>Оставить без изменений</Button
        ><Button onclick={onLater}>Разобрать позже</Button>
      </div>
    {/if}
  {/if}
</Surface>
