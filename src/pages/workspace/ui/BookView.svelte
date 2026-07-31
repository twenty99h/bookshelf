<script lang="ts">
  import { resolve } from "$app/paths";
  import { BookOpen, MoreHorizontal } from "@lucide/svelte";
  import { Button } from "@/shared/ui";
  import type { Book, LibraryAction, LibraryState } from "@/shared/api";
  let {
    library,
    selectedBook,
    bookStatus,
    onRun,
    onDelete,
  }: {
    library: LibraryState;
    selectedBook: Book | null;
    bookStatus: (book: Book) => string;
    onRun: (action: LibraryAction, message?: string) => Promise<boolean>;
    onDelete: () => void;
  } = $props();
</script>

{#if selectedBook}
  <div class="grid grid-cols-[180px_minmax(0,1fr)_260px] gap-8 max-[1280px]:grid-cols-[140px_minmax(0,1fr)]">
    <div
      class="grid h-60 place-items-center rounded-r-xl border border-white/10 bg-night/50 text-5xl font-semibold text-amber shadow-[inset_9px_0_0_#d6a24a]"
    >
      {selectedBook.title[0]}
    </div>
    <div>
      <p class="font-mono text-xs uppercase tracking-[.14em] text-iris">{bookStatus(selectedBook)}</p>
      <h2 class="mt-3 text-4xl font-semibold tracking-[-.035em]">{selectedBook.title}</h2>
      <p class="mt-4 text-mist-dim">
        PDF · {selectedBook.pageCount} страниц · {selectedBook.hasTextLayer
          ? "есть оглавление и текстовый слой"
          : "без текстового слоя — доступна ручная заметка к странице"}
      </p>
      <div class="mt-7 flex gap-3">
        <a
          class="inline-flex min-h-11 items-center gap-2 rounded-md bg-iris-strong px-5 font-semibold text-white no-underline"
          href={resolve("/reader/[bookId]", { bookId: selectedBook.id })}
          ><BookOpen class="size-4" />Продолжить чтение</a
        ><Button onclick={() => onRun({ kind: "activateStudy", bookId: selectedBook.id }, "Книга стала активной")}
          >Сделать активной</Button
        ><button aria-label="Другие действия" class="grid size-11 place-items-center rounded-md border border-white/10"
          ><MoreHorizontal /></button
        >
      </div>
      <div class="mt-4 flex gap-4 text-sm">
        {#if selectedBook.archived}<button
            class="text-iris"
            onclick={() => onRun({ kind: "restoreBook", bookId: selectedBook.id }, "Книга возвращена из архива")}
            >Вернуть из архива</button
          >{:else}<button
            class="text-mist-dim"
            onclick={() => onRun({ kind: "archiveBook", bookId: selectedBook.id }, "Книга перемещена в архив")}
            >Архивировать</button
          >{/if}
        {#if selectedBook.studyStatus === "completed"}<button
            class="text-iris"
            onclick={() => onRun({ kind: "startRepeatStudy", bookId: selectedBook.id }, "Начат новый цикл изучения")}
            >Начать повторное изучение</button
          >{/if}
        <button class="text-danger" onclick={() => onDelete()}>Удалить навсегда</button>
      </div>
    </div>
    <aside class="rounded-xl border border-white/8 bg-slate p-5 max-[1280px]:col-span-2">
      <p class="font-mono text-xs uppercase text-mist-dim">Прогресс чтения</p>
      <div class="mt-4 flex items-end justify-between">
        <span class="font-mono text-3xl">{selectedBook.reading.page}</span><span class="text-sm text-mist-dim"
          >последняя</span
        >
      </div>
      <div class="mt-3 h-1.5 rounded-full bg-night"><div class="h-full w-[56%] rounded-full bg-iris"></div></div>
      <p class="mt-5 text-sm text-mist-dim">
        Самая дальняя позиция <span class="font-mono text-mist">312</span>. Возврат к источнику её не уменьшает.
      </p>
    </aside>
  </div>
  <nav class="mt-9 flex gap-7 border-b border-white/8" aria-label="Разделы книги">
    {#each ["Обзор", "Черновики", "Идеи", "Практика"] as tab (tab)}<button
        class="border-b-2 border-transparent px-1 pb-3 text-sm text-mist-dim first:border-iris first:text-mist"
        >{tab}</button
      >{/each}
  </nav>
  <section class="mt-6 grid grid-cols-3 gap-5">
    <article class="col-span-2 rounded-xl border border-white/8 bg-slate p-6">
      <h2 class="text-xl font-semibold">Продолжить с контекстом</h2>
      <p class="mt-2 text-mist-dim">Глава 5 · Репликация · страница {selectedBook.reading.page}</p>
      <div class="mt-6 border-l-2 border-amber pl-4">
        <small class="font-mono uppercase text-amber">Последний источник</small>
        <p class="mt-2 leading-6">
          {library!.drafts.find((draft) => draft.bookId === selectedBook.id)?.excerpt ??
            "У этой книги пока нет сохранённых источников."}
        </p>
      </div>
    </article>
    <article class="rounded-xl border border-white/8 bg-slate p-6">
      <p class="font-mono text-xs uppercase text-mist-dim">Прогресс изучения</p>
      <dl class="mt-4 grid gap-3 text-sm">
        <div class="flex">
          <dt>Разобрано</dt>
          <dd class="ml-auto font-mono">7</dd>
        </div>
        <div class="flex">
          <dt>Идей</dt>
          <dd class="ml-auto font-mono">{library!.ideas.filter((idea) => idea.bookId === selectedBook.id).length}</dd>
        </div>
        <div class="flex">
          <dt>Восстановлений</dt>
          <dd class="ml-auto font-mono">2</dd>
        </div>
        <div class="flex">
          <dt>Применений</dt>
          <dd class="ml-auto font-mono">1</dd>
        </div>
      </dl>
      <a
        href={resolve("/library/[bookId]/complete", { bookId: selectedBook.id })}
        class="mt-6 inline-flex text-sm font-semibold text-iris no-underline">Подвести итог изучения</a
      >
    </article>
  </section>
{/if}
