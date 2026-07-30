<script lang="ts">
  import { Check, ChevronDown } from "@lucide/svelte";
  import { Label, Select, useId } from "bits-ui";

  type SelectOption = { value: string; label: string; disabled?: boolean };

  let {
    id = useId(),
    label,
    value = $bindable(""),
    options,
    placeholder = "Выберите значение",
    disabled = false,
    onValueChange,
  }: {
    id?: string;
    label: string;
    value?: string;
    options: SelectOption[];
    placeholder?: string;
    disabled?: boolean;
    onValueChange?: (value: string) => void;
  } = $props();

  const labelId = $derived(`${id}-label`);
  function setValue(next: string) {
    value = next;
    onValueChange?.(next);
  }
</script>

<div class="grid gap-1.5">
  <Label.Root id={labelId} class="text-xs font-bold text-slate-600">{label}</Label.Root>
  <Select.Root type="single" items={options} bind:value={() => value, setValue} {disabled}>
    <Select.Trigger
      aria-labelledby={labelId}
      class="flex min-h-11 w-full items-center gap-2 rounded-lg border border-slate-300 bg-paper-raised px-3 py-2.5 text-left text-ink outline-none focus-visible:border-leaf focus-visible:ring-3 focus-visible:ring-focus disabled:opacity-50"
    >
      <Select.Value {placeholder} />
      <ChevronDown class="ml-auto size-4" aria-hidden="true" />
    </Select.Trigger>
    <Select.Portal>
      <Select.Content
        class="z-50 max-h-[min(20rem,var(--bits-select-content-available-height))] w-[var(--bits-select-anchor-width)] overflow-auto rounded-lg border border-rule bg-paper-raised p-1 shadow-paper"
        sideOffset={4}
      >
        <Select.Viewport>
          {#each options as option (option.value)}
            <Select.Item
              value={option.value}
              label={option.label}
              disabled={option.disabled}
              class="flex min-h-10 cursor-default items-center rounded-md px-3 py-2 text-sm outline-none data-[highlighted]:bg-leaf-soft data-[disabled]:opacity-50"
            >
              {#snippet children({ selected })}
                {option.label}
                {#if selected}<Check class="ml-auto size-4" aria-hidden="true" />{/if}
              {/snippet}
            </Select.Item>
          {/each}
        </Select.Viewport>
      </Select.Content>
    </Select.Portal>
  </Select.Root>
</div>
