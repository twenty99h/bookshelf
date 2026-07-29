<script lang="ts">
  let { id, label, value = $bindable(""), placeholder = "", disabled = false, required = false }: { id: string; label: string; value?: string; placeholder?: string; disabled?: boolean; required?: boolean } = $props();
</script>
<label class="mb-1.5 block text-xs font-bold text-[#4d5861]" for={id}>{label}</label>
<textarea {id} bind:value {placeholder} {disabled} {required} class="min-h-[84px] w-full resize-y rounded-lg border border-[#cfd1cd] bg-paper-raised px-3 py-2.5 text-ink outline-none focus:border-[#697c39] focus:ring-3 focus:ring-focus disabled:opacity-50"></textarea>
