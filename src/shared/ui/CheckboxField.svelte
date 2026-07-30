<script lang="ts">
  import { Check } from "@lucide/svelte";
  import { Checkbox, Label, useId } from "bits-ui";

  let {
    id = useId(),
    label,
    checked = $bindable(false),
    disabled = false,
    onCheckedChange,
  }: {
    id?: string;
    label: string;
    checked?: boolean;
    disabled?: boolean;
    onCheckedChange?: (checked: boolean) => void;
  } = $props();

  function setChecked(next: boolean) {
    checked = next;
    onCheckedChange?.(next);
  }
</script>

<div class="flex items-center gap-2">
  <Checkbox.Root
    {id}
    bind:checked={() => checked, setChecked}
    {disabled}
    class="flex size-5 shrink-0 items-center justify-center rounded border border-white/25 bg-night/45 text-white outline-none data-[state=checked]:border-iris-strong data-[state=checked]:bg-iris-strong focus-visible:ring-3 focus-visible:ring-focus disabled:opacity-50"
  >
    {#snippet children({ checked })}
      {#if checked}<Check class="size-4" strokeWidth={3} aria-hidden="true" />{/if}
    {/snippet}
  </Checkbox.Root>
  <Label.Root for={id} class="text-sm font-medium text-ink">{label}</Label.Root>
</div>
