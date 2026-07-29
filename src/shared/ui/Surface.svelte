<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    children,
    class: className = "",
    ariaLabel,
  }: { children: Snippet; class?: string; ariaLabel?: string } = $props();
</script>

<section
  aria-label={ariaLabel}
  class={`rounded-[11px] border border-rule bg-paper-raised p-6 shadow-paper ${className}`}
>
  {@render children()}
</section>
