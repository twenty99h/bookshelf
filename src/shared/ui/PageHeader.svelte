<script lang="ts">
  import type { Snippet } from "svelte";
  let { eyebrow, title, actions }: { eyebrow: string; title: string; actions?: Snippet } = $props();
</script>

<header class="mb-6 flex flex-wrap items-end justify-between gap-4 sm:mb-[26px]">
  <div>
    <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">{eyebrow}</p>
    <h1 class="m-0 font-display text-[clamp(32px,4vw,48px)] font-medium leading-[1.05] tracking-[-.035em]">{title}</h1>
  </div>
  {#if actions}{@render actions()}{/if}
</header>
