<script lang="ts">
  import { Label } from "bits-ui";

  let {
    id,
    label,
    value = $bindable(""),
    placeholder = "",
    maxlength,
    disabled = false,
    type = "text",
  }: {
    id: string;
    label: string;
    value?: string;
    placeholder?: string;
    maxlength?: number;
    disabled?: boolean;
    type?: "text" | "password" | "search";
  } = $props();
</script>

<Label.Root class="block text-[13px] font-bold text-ink" for={id}>{label}</Label.Root>
<input
  {id}
  bind:value
  {placeholder}
  {maxlength}
  {disabled}
  {type}
  class="min-h-11 w-full min-w-0 rounded-lg border border-[#cfd1c9] bg-[#fafaf8] px-3 py-2.5 text-ink outline-none transition-shadow placeholder:text-[#91938b] focus:border-leaf focus:ring-3 focus:ring-focus disabled:cursor-not-allowed disabled:opacity-50"
/>
