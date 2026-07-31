<script lang="ts">
  import "@/app/styles/global.css";
  import type { Snippet } from "svelte";
  import AppShell from "@/app/ui/AppShell.svelte";

  let { children }: { children: Snippet } = $props();
</script>

<AppShell>{@render children()}</AppShell>
