<script lang="ts">
  import type { DraftNote, Idea } from "@/shared/api";
  import DraftCard from "./DraftCard.svelte";

  let {
    drafts,
    ideas,
    bookTitle,
    onCreate,
    onAttach,
    onExport,
    onDiscard,
  }: {
    drafts: DraftNote[];
    ideas: Idea[];
    bookTitle: (bookId: string) => string;
    onCreate: (draft: DraftNote, formulation: string) => Promise<boolean>;
    onAttach: (draftId: string, ideaId: string) => Promise<void>;
    onExport: (draftId: string) => Promise<void>;
    onDiscard: (draftId: string) => Promise<void>;
  } = $props();
</script>

{#if drafts.length === 0}
  <section
    class="grid justify-items-center rounded-[14px] border border-rule bg-paper-raised px-6 py-[42px] text-center shadow-paper [&>p]:max-w-[590px] [&>p]:leading-[1.65] [&>p]:text-ink-muted"
  >
    <h2>Очередь разобрана</h2>
    <p>Новые фрагменты можно сохранить, не выходя из просмотрщика.</p>
  </section>
{:else}
  <section class="grid gap-3.5">
    {#each drafts as draft (draft.id)}
      <DraftCard
        {draft}
        bookTitle={bookTitle(draft.bookId)}
        {ideas}
        onCreate={(formulation) => onCreate(draft, formulation)}
        onAttach={(ideaId) => onAttach(draft.id, ideaId)}
        onExport={() => onExport(draft.id)}
        onDiscard={() => onDiscard(draft.id)}
      />
    {/each}
  </section>
{/if}
