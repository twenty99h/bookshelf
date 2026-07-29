<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    children,
    tone = "neutral",
  }: {
    children: Snippet;
    tone?: "neutral" | "success" | "danger";
  } = $props();

  const tones = {
    neutral: "text-ink-muted",
    success: "text-success",
    danger: "text-danger",
  };
</script>

<p class={`mt-2 text-[13px] leading-5 ${tones[tone]}`} role={tone === "danger" ? "alert" : "status"}>
  {@render children()}
</p>
