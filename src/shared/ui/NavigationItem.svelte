<script lang="ts">
  let { label, active = false, count, onclick }: { label: string; active?: boolean; count?: number; onclick: () => void } = $props();
</script>

<button
  class={`flex min-h-11 shrink-0 items-center justify-between gap-3 rounded-lg border-0 px-3 py-2.5 text-left transition-colors focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-[#9eb66a] ${active ? "bg-spine-raised text-white" : "bg-transparent text-spine-muted hover:bg-spine-raised hover:text-white"}`}
  aria-current={active ? "page" : undefined}
  {onclick}
>
  <span>{label}</span>
  {#if count !== undefined}<small class="min-w-6 rounded-full bg-[#3a4751] px-1.5 py-0.5 text-center font-mono text-[11px]">{count}</small>{/if}
</button>
