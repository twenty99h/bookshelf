<script lang="ts">
  let {
    id,
    label,
    ariaLabel,
    value = $bindable(""),
    placeholder = "",
    disabled = false,
    required = false,
  }: {
    id: string;
    label?: string;
    ariaLabel?: string;
    value?: string;
    placeholder?: string;
    disabled?: boolean;
    required?: boolean;
  } = $props();
</script>

{#if label}<label class="mb-1.5 block text-xs font-bold text-[#4d5861]" for={id}>{label}</label>{/if}
<textarea
  {id}
  aria-label={ariaLabel}
  bind:value
  {placeholder}
  {disabled}
  {required}
  class="min-h-[84px] w-full resize-y rounded-lg border border-[#cfd1cd] bg-paper-raised px-3 py-2.5 text-ink outline-none focus:border-[#697c39] focus:ring-3 focus:ring-focus disabled:opacity-50"
></textarea>
