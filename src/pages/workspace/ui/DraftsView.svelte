<script lang="ts">
  import { resolve } from "$app/paths";
  import { Check } from "@lucide/svelte";
  import { Button, TextArea } from "@/shared/ui";
  import type { LibraryAction, LibraryState } from "@/shared/api";
  type Draft = LibraryState["drafts"][number];
  let {
    library,
    focusedDraft,
    mode = $bindable(),
    formulation = $bindable(),
    busy,
    onResolve,
    onAttach,
    onRun,
    onExport,
  }: {
    library: LibraryState;
    focusedDraft: Draft | null;
    mode: "focus" | "list";
    formulation: string;
    busy: boolean;
    onResolve: () => Promise<void>;
    onAttach: () => Promise<void>;
    onRun: (action: LibraryAction, message?: string) => Promise<boolean>;
    onExport: () => Promise<void>;
  } = $props();
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
    ><button
      class="rounded px-3 py-1.5 text-sm data-[active=true]:bg-iris/15"
      data-active={mode === "list"}
      onclick={() => (mode = "list")}>Все заметки</button
    >
  </div>
</div>
{#if !focusedDraft}<section
    class="grid min-h-[55vh] place-items-center rounded-xl border border-white/8 bg-slate text-center"
  >
    <div>
      <Check class="mx-auto size-9 text-success" />
      <h2 class="mt-4 text-2xl font-semibold">Всё разобрано</h2>
      <p class="mt-2 text-mist-dim">Можно вернуться к чтению или открыть знания.</p>
    </div>
  </section>
{:else if mode === "list"}<section class="rounded-xl border border-white/8 bg-slate">
    {#each library!.drafts as draft (draft.id)}<article
        class="grid grid-cols-[180px_1fr_160px] gap-5 border-b border-white/8 p-5"
      >
        <span class="text-sm text-mist-dim"
          >{draft.section}<small class="block font-mono text-amber">стр. {draft.page}</small></span
        >
        <p>{draft.excerpt}</p>
        <button class="text-right text-sm font-semibold text-iris" onclick={() => (mode = "focus")}>Разобрать</button>
      </article>{/each}
  </section>
{:else}<section class="grid grid-cols-[minmax(0,.9fr)_minmax(420px,1.1fr)] gap-6">
    <article class="rounded-xl border border-white/8 bg-night/35 p-7">
      <p class="font-mono text-xs uppercase tracking-[.14em] text-amber">
        Источник · {focusedDraft.section} · стр. {focusedDraft.page}
      </p>
      <blockquote class="mt-6 border-l-2 border-amber pl-5 text-lg leading-8">{focusedDraft.excerpt}</blockquote>
      <p class="mt-5 text-sm leading-6 text-mist-dim">{focusedDraft.context}</p>
      <a
        href={resolve("/reader/[bookId]", { bookId: focusedDraft.bookId })}
        class="mt-6 inline-flex text-sm text-amber no-underline">Открыть источник в книге</a
      >
    </article>
    <article class="rounded-xl border border-white/8 bg-slate p-7">
      <p class="font-mono text-xs uppercase tracking-[.14em] text-iris">
        Решение {library!.drafts.length > 1 ? `· ещё ${library!.drafts.length - 1}` : ""}
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
        <Button variant="primary" onclick={onResolve} disabled={!formulation.trim() || busy}>Создать идею</Button
        ><Button onclick={onAttach}>Присоединить к идее</Button><Button
          onclick={() => onRun({ kind: "deferDraft", draftId: focusedDraft.id }, "Заметка отложена")}>Отложить</Button
        ><Button onclick={onExport}>Экспортировать</Button><Button
          onclick={() => onRun({ kind: "discardDraft", draftId: focusedDraft.id }, "Черновая заметка удалена")}
          >Удалить</Button
        >
      </div>
    </article>
  </section>{/if}
