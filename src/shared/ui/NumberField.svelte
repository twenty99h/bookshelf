<script lang="ts">
  import { Label } from "bits-ui";

  let {
    id,
    label,
    ariaLabel,
    value = $bindable(0),
    min,
    max,
    step,
    disabled = false,
    onChange,
  }: {
    id: string;
    label?: string;
    ariaLabel?: string;
    value?: number;
    min?: number;
    max?: number;
    step?: number;
    disabled?: boolean;
    onChange?: (value: number) => void;
  } = $props();
</script>

{#if label}<Label.Root class="block text-[13px] font-bold text-ink" for={id}>{label}</Label.Root>{/if}
<input
  {id}
  aria-label={ariaLabel}
  type="number"
  bind:value
  {min}
  {max}
  {step}
  {disabled}
  onchange={() => onChange?.(value)}
  class="min-h-11 w-full min-w-0 rounded-lg border border-[#cfd1c9] bg-[#fafaf8] px-3 py-2.5 text-ink outline-none transition-shadow focus:border-leaf focus:ring-3 focus:ring-focus disabled:cursor-not-allowed disabled:opacity-50"
/>
