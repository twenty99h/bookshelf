<script lang="ts">
  import { Search } from "@lucide/svelte";
  import { Button, DialogModal, TextField } from "@/shared/ui";

  type PaletteResult = { id: string; kind: string; title: string; context: string };

  let {
    open = $bindable(),
    query = $bindable(),
    results,
    onSearch,
    onOpenResult,
  }: {
    open: boolean;
    query: string;
    results: PaletteResult[];
    onSearch: () => Promise<void>;
    onOpenResult: (result: PaletteResult) => void;
  } = $props();
</script>

<DialogModal
  bind:open
  title="Быстрый переход"
  description="Найдите книгу, идею, тему, черновик или материал для передачи."
>
  {#snippet trigger()}<span class="sr-only">Открыть быстрый переход</span>{/snippet}
  <form
    class="grid gap-3"
    onsubmit={(event) => {
      event.preventDefault();
      void onSearch();
    }}
  >
    <TextField id="command-search" label="Поиск" bind:value={query} placeholder="Название или формулировка" />
    <Button type="submit">Найти</Button>
  </form>
  <div class="grid gap-1" aria-live="polite">
    {#if query && results.length === 0}<p class="text-sm text-mist-dim">
        Совпадений нет. Измените запрос, введённый текст сохранён.
      </p>{/if}
    {#each results as result (`${result.kind}-${result.id}`)}
      <button
        class="flex items-center gap-3 rounded-lg border border-white/8 bg-slate p-3 text-left hover:border-iris/50"
        onclick={() => onOpenResult(result)}
      >
        <Search class="size-4 text-iris" /><span
          ><b class="line-clamp-1">{result.title}</b><small class="block text-mist-dim">{result.context}</small></span
        >
      </button>
    {/each}
  </div>
</DialogModal>
