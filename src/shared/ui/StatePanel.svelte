<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    children,
    tone = "neutral",
    live = false,
  }: {
    children: Snippet;
    tone?: "neutral" | "danger";
    live?: boolean;
  } = $props();
</script>

<section
  class={`rounded-[14px] border border-rule bg-paper-raised p-7 shadow-paper ${tone === "danger" ? "text-danger" : "text-ink-muted"}`}
  role={tone === "danger" ? "alert" : undefined}
  aria-live={live ? "polite" : undefined}
>
  {@render children()}
</section>
