<script lang="ts">
  let {
    id,
    label,
    ariaLabel,
    value = $bindable(""),
    placeholder = "",
    disabled = false,
    required = false,
    onValueChange,
  }: {
    id: string;
    label?: string;
    ariaLabel?: string;
    value?: string;
    placeholder?: string;
    disabled?: boolean;
    required?: boolean;
    onValueChange?: (value: string) => void;
  } = $props();
</script>

{#if label}<label class="mb-1.5 block text-xs font-bold text-mist-dim" for={id}>{label}</label>{/if}
<textarea
  {id}
  aria-label={ariaLabel}
  bind:value
  {placeholder}
  {disabled}
  {required}
  oninput={() => onValueChange?.(value)}
  class="min-h-[84px] w-full resize-y rounded-lg border border-white/15 bg-night/45 px-3 py-2.5 text-ink outline-none placeholder:text-mist-faint focus:border-iris focus:ring-3 focus:ring-focus disabled:opacity-50"
></textarea>
