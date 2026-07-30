<script lang="ts">
  import { Button, CheckboxField, TextArea } from "@/shared/ui";

  let {
    onComplete,
  }: {
    onComplete: (result: {
      situation: string;
      action: string;
      result: string;
      conclusion: string;
      successful: boolean;
    }) => Promise<void>;
  } = $props();

  let situation = $state("");
  let action = $state("");
  let result = $state("");
  let conclusion = $state("");
  let successful = $state(false);

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    await onComplete({ situation, action, result, conclusion, successful });
  }
</script>

<form
  class="grid content-start gap-1.5 rounded-[11px] border border-rule bg-paper-raised p-6 shadow-paper [&_h2]:mb-2 [&_h2]:font-display [&_h2]:text-[25px] [&_h2]:font-medium [&_h2]:leading-tight"
  onsubmit={submit}
>
  <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">Практический эксперимент</p>
  <h2>Зафиксировать результат</h2>
  <TextArea id="situation" label="Ситуация" bind:value={situation} />
  <TextArea id="action-taken" label="Действие" bind:value={action} />
  <TextArea id="observed" label="Наблюдаемый результат" bind:value={result} />
  <TextArea id="conclusion" label="Мой вывод" bind:value={conclusion} />
  <CheckboxField label="Результат оказался положительным" bind:checked={successful} />
  <Button type="submit">Завершить эксперимент</Button>
</form>
