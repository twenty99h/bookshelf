<script lang="ts">
  import { onMount } from "svelte";
  import { Button, StatePanel, StatusMessage, TextField } from "@/shared/ui";
  import {
    commandErrorMessage,
    loadLibrary,
    saveWorkspaceNote,
    type LibraryState,
  } from "../../../platform/commands/library";

  let library = $state<LibraryState | null>(null);
  let note = $state("");
  let loading = $state(true);
  let saving = $state(false);
  let saved = $state(false);
  let error = $state("");

  onMount(async () => {
    try {
      library = await loadLibrary();
      note = library.workspaceNote;
    } catch (cause) {
      error = commandErrorMessage(cause);
    } finally {
      loading = false;
    }
  });

  async function saveNote(event: SubmitEvent) {
    event.preventDefault();
    saving = true;
    saved = false;
    error = "";
    try {
      library = await saveWorkspaceNote(note);
      saved = true;
    } catch (cause) {
      error = commandErrorMessage(cause);
    } finally {
      saving = false;
    }
  }
</script>

<svelte:head>
  <title>Bookshelf — личная библиотека</title>
  <meta
    name="description"
    content="Локальное пространство для системного изучения технических книг"
  />
</svelte:head>

<div class="grid min-h-screen grid-cols-1 md:grid-cols-[248px_1fr]">
  <aside class="flex min-h-[184px] flex-col bg-spine px-5 py-6 text-[#f6f3e9] md:min-h-screen md:px-[22px] md:py-[30px]">
    <div class="flex items-center gap-2.5 font-display text-lg font-semibold tracking-[0.02em]">
      <span class="grid size-[34px] place-items-center rounded-lg bg-[#c8d89a] text-[#26301f]" aria-hidden="true">B</span>
      Bookshelf
    </div>
    <nav class="mt-6 grid gap-2 md:mt-12" aria-label="Основная навигация">
      <a class="rounded-lg bg-spine-raised px-3 py-2.5 text-sm text-white outline-none focus-visible:ring-3 focus-visible:ring-[#8aa06f]" href="/">Библиотека</a>
      <span class="rounded-lg px-3 py-2.5 text-sm text-spine-muted">Очередь разбора <small class="float-right rounded-full bg-[#485044] px-2 py-0.5">0</small></span>
      <span class="rounded-lg px-3 py-2.5 text-sm text-spine-muted">Идеи</span>
    </nav>
    <p class="mt-auto hidden text-xs text-[#9da496] md:block"><i class="mr-1.5 inline-block size-[7px] rounded-full bg-[#accb74]"></i> Данные хранятся локально</p>
  </aside>

  <main class="w-full max-w-[1150px] px-5 py-8 sm:px-10 md:px-[clamp(40px,7vw,92px)] md:py-[54px]">
    <header class="mb-[30px] flex flex-wrap items-end justify-between gap-4">
      <div>
        <p class="m-0 text-xs font-bold uppercase tracking-[0.1em] text-[#68705f]">Личная библиотека</p>
        <h1 class="mt-1 font-display text-4xl font-medium leading-[1.1] sm:text-[42px]">Ваши книги</h1>
      </div>
      <Button variant="primary" disabled>Импортировать PDF</Button>
    </header>

    {#if loading}
      <StatePanel live>Открываем библиотеку…</StatePanel>
    {:else if error && !library}
      <StatePanel tone="danger">
        <strong>Библиотека не открылась</strong>
        <p class="mt-2 leading-6">{error}</p>
      </StatePanel>
    {:else if library}
      {#if library.books.length === 0}
        <section class="rounded-[14px] border border-rule bg-paper-raised px-6 py-12 text-center shadow-paper sm:px-10 sm:py-[54px]">
          <div class="mx-auto mb-[22px] grid size-[58px] place-items-center rounded-2xl bg-leaf-soft text-3xl text-[#61734c]" aria-hidden="true">▥</div>
          <p class="m-0 text-xs font-bold uppercase tracking-[0.1em] text-[#68705f]">Здесь появится ваша коллекция</p>
          <h2 class="my-2 font-display text-2xl font-medium leading-tight sm:text-[25px]">Начните с одной важной книги</h2>
          <p class="mx-auto mb-0 max-w-[540px] leading-6 text-ink-muted">Bookshelf хранит книги и рабочее состояние на этом компьютере. Импорт PDF станет доступен на следующем шаге.</p>
        </section>
      {/if}

      <section class="mt-5 grid gap-5 rounded-[14px] border border-rule bg-paper-raised px-6 py-7 shadow-paper sm:px-[34px] lg:grid-cols-[minmax(220px,.75fr)_minmax(320px,1fr)] lg:gap-[42px]">
        <div>
          <p class="m-0 text-xs font-bold uppercase tracking-[0.1em] text-[#68705f]">Проверка рабочего состояния</p>
          <h2 class="my-2 font-display text-2xl font-medium leading-tight sm:text-[25px]">Заметка пространства</h2>
          <p class="mb-0 leading-6 text-ink-muted">Сохраните короткую пометку — она останется здесь после перезапуска приложения.</p>
        </div>
        <form class="self-center" onsubmit={saveNote}>
          <TextField id="workspace-note" label="Пометка" bind:value={note} placeholder="Например, выбрать следующую книгу" maxlength={240} disabled={saving} />
          <div class="mt-2 flex items-center gap-3">
            <Button type="submit" disabled={saving} class="min-w-[116px]">{saving ? "Сохраняем…" : "Сохранить"}</Button>
            {#if saved}<StatusMessage tone="success">Сохранено локально</StatusMessage>{/if}
          </div>
          {#if error}<StatusMessage tone="danger">{error}</StatusMessage>{/if}
        </form>
      </section>
    {/if}
  </main>
</div>
