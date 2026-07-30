<script lang="ts">
  import { Button, TextArea, TextField } from "@/shared/ui";

  let {
    onSave,
  }: {
    onSave: (material: {
      title: string;
      problem: string;
      idea: string;
      example: string;
      result: string;
      limitations: string;
    }) => Promise<void>;
  } = $props();

  let title = $state("");
  let problem = $state("");
  let idea = $state("");
  let example = $state("");
  let result = $state("");
  let limitations = $state("");

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    await onSave({ title, problem, idea, example, result, limitations });
  }
</script>

<form
  class="grid content-start gap-1.5 rounded-[11px] border border-rule bg-paper-raised p-6 shadow-paper [&_h2]:mb-2 [&_h2]:font-display [&_h2]:text-[25px] [&_h2]:font-medium [&_h2]:leading-tight"
  onsubmit={submit}
>
  <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">Передача знания</p>
  <h2>Авторский Markdown-материал</h2>
  <TextField id="material-title" label="Название" bind:value={title} />
  <TextArea id="problem" label="Проблема" bind:value={problem} />
  <TextArea id="material-idea" label="Идея" bind:value={idea} />
  <TextArea id="example" label="Пример применения" bind:value={example} />
  <TextArea id="material-result" label="Результат" bind:value={result} />
  <TextArea id="limitations" label="Ограничения" bind:value={limitations} />
  <Button type="submit">Сохранить материал</Button>
</form>
