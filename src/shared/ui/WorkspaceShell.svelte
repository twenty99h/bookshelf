<script lang="ts">
  import type { Snippet } from "svelte";
  let { sidebar, children }: { sidebar: Snippet; children: Snippet } = $props();
</script>

<div class="grid min-h-screen grid-cols-1 min-[901px]:grid-cols-[232px_minmax(0,1fr)]">
  <aside class="static z-20 flex min-h-0 flex-col bg-spine px-[18px] py-3.5 text-white min-[901px]:sticky min-[901px]:top-0 min-[901px]:h-screen min-[901px]:px-[18px] min-[901px]:py-[26px]">
    {@render sidebar()}
  </aside>
  <main class="w-full max-w-[1260px] px-4 py-6 sm:px-7 sm:py-8 min-[901px]:px-[clamp(28px,5.4vw,74px)] min-[901px]:py-12">
    {@render children()}
  </main>
</div>
