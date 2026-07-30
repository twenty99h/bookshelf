<script lang="ts">
  import type { DraftNote, Idea } from "@/shared/api";
  import { Button, SelectField, TextArea } from "@/shared/ui";

  let {
    draft,
    bookTitle,
    ideas,
    onCreate,
    onAttach,
    onExport,
    onDiscard,
  }: {
    draft: DraftNote;
    bookTitle: string;
    ideas: Idea[];
    onCreate: (formulation: string) => Promise<boolean>;
    onAttach: (ideaId: string) => Promise<void>;
    onExport: () => Promise<void>;
    onDiscard: () => Promise<void>;
  } = $props();

  let formulation = $state("");
  let attachIdeaId = $state("");

  async function createIdea() {
    if (await onCreate(formulation)) formulation = "";
  }

  async function attach() {
    if (!attachIdeaId) return;
    await onAttach(attachIdeaId);
    attachIdeaId = "";
  }
</script>

<article class="rounded-[11px] border border-rule bg-paper-raised p-6 shadow-paper">
  <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">
    {bookTitle} · {draft.section} · стр. {draft.page}
  </p>
  <blockquote>{draft.excerpt}</blockquote>
  {#if draft.comment}<p>{draft.comment}</p>{/if}
  <TextArea id={`idea-${draft.id}`} label="Самостоятельная формулировка" bind:value={formulation} />
  <div class="mt-2.5 flex flex-wrap items-center gap-[9px]">
    <Button variant="primary" onclick={createIdea}>Создать идею</Button>
    <SelectField
      label="Идея для присоединения"
      bind:value={attachIdeaId}
      placeholder="Выберите идею"
      options={ideas.map((idea) => ({ value: idea.id, label: idea.formulation }))}
    />
    <Button disabled={!attachIdeaId} onclick={attach}>Присоединить</Button>
    <Button onclick={onExport}>Экспортировать</Button>
    <Button onclick={onDiscard}>Удалить</Button>
  </div>
</article>
