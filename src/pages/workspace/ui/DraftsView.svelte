<script lang="ts">
  import { resolve } from "$app/paths";
  import { Check } from "@lucide/svelte";
  import { Button, SelectField, TextArea } from "@/shared/ui";
  import type { LibraryState } from "@/shared/api";

  type Draft = LibraryState["drafts"][number];

  let {
    library,
    focusedDraft,
    bookFilterId,
    busy,
    onSelectDraft,
    onResolve,
    onAttach,
    onDefer,
    onDiscard,
    onExport,
  }: {
    library: LibraryState;
    focusedDraft: Draft | null;
    bookFilterId: string;
    busy: boolean;
    onSelectDraft: (draftId: string) => void;
    onResolve: (formulation: string) => Promise<void>;
    onAttach: (ideaId: string) => Promise<void>;
    onDefer: () => Promise<void>;
    onDiscard: () => Promise<void>;
    onExport: () => Promise<void>;
  } = $props();

  let mode = $state<"focus" | "list">("focus");
  let formulation = $state("");
  let attachIdeaId = $state("");
  const compatibleIdeas = $derived(library.ideas.filter((idea) => idea.bookId === focusedDraft?.bookId));
  const visibleDrafts = $derived(library.drafts.filter((draft) => !bookFilterId || draft.bookId === bookFilterId));

  function selectDraft(draftId: string) {
    onSelectDraft(draftId);
    formulation = "";
    attachIdeaId = "";
    mode = "focus";
  }

  async function resolveDraft() {
    await onResolve(formulation);
    formulation = "";
  }

  async function attachDraft() {
    if (!attachIdeaId) return;
    await onAttach(attachIdeaId);
    attachIdeaId = "";
  }
</script>

<div class="mb-6 flex items-center justify-between">
  <p class="max-w-2xl text-sm leading-6 text-mist-dim">
    Одно решение за раз. Полный список нужен только когда вы ищете конкретный материал.
  </p>
  <div class="flex rounded-md border border-white/8 p-1">
    <button
      class="rounded px-3 py-1.5 text-sm data-[active=true]:bg-iris/15"
      data-active={mode === "focus"}
      onclick={() => (mode = "focus")}>Разбор</button
    >
    <button
      class="rounded px-3 py-1.5 text-sm data-[active=true]:bg-iris/15"
      data-active={mode === "list"}
      onclick={() => (mode = "list")}>Все заметки</button
    >
  </div>
</div>

{#if !focusedDraft}
  <section class="grid min-h-[55vh] place-items-center rounded-xl border border-white/8 bg-slate text-center">
    <div>
      <Check class="mx-auto size-9 text-success" />
      <h2 class="mt-4 text-2xl font-semibold">Всё разобрано</h2>
      <p class="mt-2 text-mist-dim">Можно вернуться к чтению или открыть знания.</p>
    </div>
  </section>
{:else if mode === "list"}
  <section class="rounded-xl border border-white/8 bg-slate">
    {#each visibleDrafts as draft (draft.id)}
      <article class="grid grid-cols-[180px_1fr_160px] gap-5 border-b border-white/8 p-5">
        <span class="text-sm text-mist-dim"
          >{draft.section}<small class="block font-mono text-amber">стр. {draft.page}</small></span
        >
        <p>{draft.excerpt}</p>
        <button class="text-right text-sm font-semibold text-iris" onclick={() => selectDraft(draft.id)}
          >Разобрать</button
        >
      </article>
    {/each}
  </section>
{:else}
  <section class="grid grid-cols-[minmax(0,.9fr)_minmax(420px,1.1fr)] gap-6 max-[1280px]:grid-cols-1">
    <article class="rounded-xl border border-white/8 bg-night/35 p-7">
      <p class="font-mono text-xs uppercase tracking-[.14em] text-amber">Источники · {focusedDraft.section}</p>
      <div class="mt-5 grid gap-4">
        {#each focusedDraft.fragments.length ? focusedDraft.fragments : [{ page: focusedDraft.page, excerpt: focusedDraft.excerpt, context: focusedDraft.context }] as fragment (`${fragment.page}-${fragment.excerpt}`)}
          <blockquote class="border-l-2 border-amber pl-5">
            <p class="text-lg leading-8">{fragment.excerpt}</p>
            <footer class="mt-2 font-mono text-xs text-amber">стр. {fragment.page}</footer>
            {#if fragment.context}<p class="mt-2 text-sm leading-6 text-mist-dim">{fragment.context}</p>{/if}
          </blockquote>
        {/each}
      </div>
      <a
        href={resolve(`/reader/${encodeURIComponent(focusedDraft.bookId)}?sourcePage=${focusedDraft.page}`)}
        class="mt-6 inline-flex text-sm text-amber no-underline">Открыть источник в книге</a
      >
    </article>
    <article class="rounded-xl border border-white/8 bg-slate p-7">
      <p class="font-mono text-xs uppercase tracking-[.14em] text-iris">
        Решение {visibleDrafts.length > 1 ? `· ещё ${visibleDrafts.length - 1}` : ""}
      </p>
      <h2 class="mt-3 text-2xl font-semibold">Сформулируйте самостоятельную идею</h2>
      <p class="mt-2 text-sm leading-6 text-mist-dim">Цитата останется источником, а не заменит вашу мысль.</p>
      <div class="mt-6">
        <TextArea
          id="draft-formulation"
          label="Моя формулировка"
          bind:value={formulation}
          placeholder="Что это утверждение меняет в ваших решениях?"
        />
      </div>
      <div class="mt-5 flex flex-wrap gap-2">
        <Button variant="primary" onclick={resolveDraft} disabled={!formulation.trim() || busy}>Создать идею</Button>
        <Button onclick={onDefer}>Отложить</Button>
        <Button onclick={onExport}>Экспортировать</Button>
        <Button onclick={onDiscard}>Удалить</Button>
      </div>
      <div class="mt-7 grid gap-3 border-t border-white/8 pt-5">
        <SelectField
          label="Идея этой книги"
          bind:value={attachIdeaId}
          options={compatibleIdeas.map((idea) => ({ value: idea.id, label: idea.formulation }))}
          placeholder="Выберите идею"
        />
        <Button disabled={!attachIdeaId} onclick={attachDraft}>Присоединить к выбранной идее</Button>
      </div>
    </article>
  </section>
{/if}
