<script lang="ts">
  import { resolve } from "$app/paths";
  import { Plus } from "@lucide/svelte";
  import { Button, SelectField } from "@/shared/ui";
  import type { Book, LibraryState } from "@/shared/api";
  let {
    library,
    books,
    filter = $bindable(),
    sort = $bindable(),
    onImport,
    bookStatus,
  }: {
    library: LibraryState;
    books: Book[];
    filter: string;
    sort: string;
    onImport: () => Promise<void>;
    bookStatus: (book: Book) => string;
  } = $props();
</script>

<div class="mb-6 flex items-end justify-between gap-5">
  <div>
    <p class="max-w-2xl text-sm leading-6 text-mist-dim">
      Книги, позиции чтения и циклы изучения. Архив остаётся фильтром, не отдельным местом.
    </p>
  </div>
  <Button variant="primary" onclick={onImport}><Plus class="mr-2 size-4" />Импортировать PDF</Button>
</div>
<div class="mb-4 grid grid-cols-[minmax(0,1fr)_220px] gap-4">
  <div class="flex flex-wrap gap-2" aria-label="Фильтры библиотеки">
    {#each [["all", "Все"], ["active", "Активные"], ["paused", "Приостановленные"], ["ready", "Готовые"], ["completed", "Завершённые"], ["archived", "Архив"]] as option (option[0])}<button
        class="rounded-md border px-3 py-2 text-sm data-[active=true]:border-iris/40 data-[active=true]:bg-iris/12 data-[active=true]:text-mist data-[active=false]:border-white/8 data-[active=false]:text-mist-dim"
        data-active={filter === option[0]}
        onclick={() => (filter = option[0] ?? "all")}>{option[1]}</button
      >{/each}
  </div>
  <SelectField
    label="Сортировка"
    value={sort}
    options={[
      { value: "recent", label: "Последнее обращение" },
      { value: "title", label: "Название" },
      { value: "progress", label: "Прогресс чтения" },
    ]}
    onValueChange={(value) => (sort = value)}
  />
</div>
<section class="overflow-hidden rounded-xl border border-white/8 bg-slate" aria-label="Книги">
  <div
    class="grid grid-cols-[54px_minmax(280px,1.5fr)_180px_130px_120px_150px] items-center gap-4 border-b border-white/8 px-5 py-3 font-mono text-[10px] uppercase tracking-[0.12em] text-mist-faint"
  >
    <span></span><span>Книга</span><span>Изучение</span><span>Позиция</span><span>Черновики</span><span></span>
  </div>
  {#each books as book (book.id)}
    <article
      class="grid min-h-[76px] grid-cols-[54px_minmax(280px,1.5fr)_180px_130px_120px_150px] items-center gap-4 border-b border-white/6 px-5 last:border-0 hover:bg-white/[0.025]"
    >
      <div
        class="grid h-12 w-9 place-items-center rounded-r border border-white/10 bg-night/50 font-semibold text-amber shadow-[inset_3px_0_0_#d6a24a]"
      >
        {book.title[0]}
      </div>
      <div>
        <a
          href={resolve("/library/[bookId]", { bookId: book.id })}
          class="font-semibold text-mist no-underline hover:text-iris">{book.title}</a
        ><small class="mt-1 block text-mist-faint"
          >PDF · {book.hasTextLayer ? "текстовый слой" : "только изображение"}</small
        >
      </div>
      <span class="text-sm text-mist-dim">{bookStatus(book)}</span><span class="font-mono text-sm"
        >стр. {book.reading.page}</span
      ><span class="font-mono text-sm">{library!.drafts.filter((draft) => draft.bookId === book.id).length}</span>
      <a
        class="inline-flex min-h-9 items-center justify-center rounded-md bg-iris-strong px-3 text-sm font-semibold text-white no-underline"
        href={resolve("/reader/[bookId]", { bookId: book.id })}>{book.reading.page > 1 ? "Продолжить" : "Открыть"}</a
      >
    </article>
  {:else}<div class="p-10 text-center">
      <h2 class="font-semibold">В этом фильтре книг нет</h2>
      <button class="mt-3 text-sm text-iris" onclick={() => (filter = "all")}>Сбросить фильтр</button>
    </div>{/each}
</section>
