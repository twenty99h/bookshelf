<script lang="ts">
  import { Trash2 } from "@lucide/svelte";
  import { CheckboxField, DialogModal, IconButton, SelectField } from ".";

  let checked = $state(false);
  let value = $state("");
  let open = $state(false);
</script>

<CheckboxField label="Восстановление" bind:checked />
<SelectField
  label="Назначение"
  bind:value
  options={[
    { value: "recall", label: "Восстановление" },
    { value: "transfer", label: "Передача" },
  ]}
/>
<p>Выбрано: {value}</p>
<DialogModal bind:open title="Справка" description="Описание доступного диалога">
  {#snippet trigger()}Открыть справку{/snippet}
  <p>Содержимое справки</p>
</DialogModal>
<IconButton label="Удалить">
  {#snippet icon()}<Trash2 class="size-5" aria-hidden="true" />{/snippet}
</IconButton>
