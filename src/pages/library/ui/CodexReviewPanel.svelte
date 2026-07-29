<script lang="ts">
  import type { ReviewKind } from "@/shared/api";
  import { Button, Eyebrow, Surface } from "@/shared/ui";

  let {
    kind, title, packageText, response, error, running, authoredFormulation,
    conclusion = $bindable(""), proposedTopic = $bindable(""),
    onStart, onCancel, onConfirmTopic, onReject, onRefine, onUnchanged, onLater,
  }: {
    kind: ReviewKind; title: string; packageText: string; response: string; error: string; running: boolean;
    authoredFormulation: string; conclusion?: string; proposedTopic?: string;
    onStart: () => void; onCancel: () => void; onConfirmTopic: () => void; onReject: () => void;
    onRefine: () => void; onUnchanged: () => void; onLater: () => void;
  } = $props();
</script>

<Surface class="mt-5" ariaLabel="Проверка через Codex">
  <Eyebrow>Передача только после подтверждения</Eyebrow>
  <h2 class="mb-2 font-display text-[25px] font-medium leading-tight">{title}</h2>
  <p>Codex получит ровно этот пакет. Полный PDF, эксперименты и другие заметки не добавляются.</p>
  <pre class="max-h-[260px] overflow-auto whitespace-pre-wrap rounded-lg border border-rule bg-[#f4f4ef] p-3.5 font-mono text-xs leading-relaxed">{packageText}</pre>
  <div class="mt-2.5 flex flex-wrap items-center gap-2"><Button variant="primary" disabled={running} onclick={onStart}>{running ? "Проверка идёт…" : "Подтвердить и отправить"}</Button><Button onclick={() => navigator.clipboard.writeText(packageText)}>Скопировать для внешнего чата</Button>{#if running}<Button onclick={onCancel}>Отменить</Button>{/if}</div>
  {#if response}<div class="my-4 whitespace-pre-wrap border-l-4 border-[#72843d] bg-leaf-soft p-4" aria-live="polite"><Eyebrow>Ответ Codex — временный</Eyebrow><p>{response}</p></div>{/if}
  {#if error}<p role="alert">{error}. Пакет можно скопировать во внешний чат; остальные функции доступны.</p>{/if}
  {#if response && !running}
    {#if kind === "topicSuggestion"}
      <label class="mb-1.5 block text-xs font-bold text-[#4d5861]" for="proposed-topic">Подтверждаемое название темы</label><input class="w-full rounded-lg border border-[#cfd1cd] bg-paper-raised px-3 py-2.5 focus:border-leaf focus:outline-none focus:ring-3 focus:ring-focus" id="proposed-topic" bind:value={proposedTopic} /><div class="mt-2.5 flex flex-wrap gap-2"><Button disabled={!proposedTopic.trim()} onclick={onConfirmTopic}>Подтвердить тему</Button><Button onclick={onReject}>Отклонить</Button></div>
    {:else if kind === "linkSuggestion"}
      <p>Предложение само ничего не меняет. Выберите идею и тип связи в форме выше, затем нажмите «Подтвердить связь».</p><Button onclick={onReject}>Отклонить</Button>
    {:else}
      <label class="mb-1.5 block text-xs font-bold text-[#4d5861]" for="review-conclusion">Мой вывод (необязательно)</label><textarea class="min-h-20 w-full resize-y rounded-lg border border-[#cfd1cd] bg-paper-raised px-3 py-2.5 focus:border-leaf focus:outline-none focus:ring-3 focus:ring-focus" id="review-conclusion" bind:value={conclusion}></textarea><div class="mt-2.5 flex flex-wrap gap-2"><Button disabled={!authoredFormulation.trim()} onclick={onRefine}>Уточнить своей формулировкой</Button><Button onclick={onUnchanged}>Оставить без изменений</Button><Button onclick={onLater}>Разобрать позже</Button></div>
    {/if}
  {/if}
</Surface>
