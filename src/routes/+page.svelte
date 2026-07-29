<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  type BookSummary = { id: string; title: string };
  type LibraryState = { books: BookSummary[]; workspaceNote: string };

  let library = $state<LibraryState | null>(null);
  let note = $state("");
  let loading = $state(true);
  let saving = $state(false);
  let saved = $state(false);
  let error = $state("");

  onMount(async () => {
    try {
      library = await invoke<LibraryState>("load_library");
      note = library.workspaceNote;
    } catch (cause) {
      error = String(cause);
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
      library = await invoke<LibraryState>("save_workspace_note", { note });
      saved = true;
    } catch (cause) {
      error = String(cause);
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

<div class="shell">
  <aside>
    <div class="brand"><span aria-hidden="true">B</span> Bookshelf</div>
    <nav aria-label="Основная навигация">
      <a class="active" href="/">Библиотека</a>
      <span>Очередь разбора <small>0</small></span>
      <span>Идеи</span>
    </nav>
    <p class="local"><i></i> Данные хранятся локально</p>
  </aside>

  <main>
    <header>
      <div>
        <p class="eyebrow">Личная библиотека</p>
        <h1>Ваши книги</h1>
      </div>
      <button class="primary" disabled>Импортировать PDF</button>
    </header>

    {#if loading}
      <section class="card status" aria-live="polite">Открываем библиотеку…</section>
    {:else if error && !library}
      <section class="card error" role="alert">
        <strong>Библиотека не открылась</strong>
        <p>{error}</p>
      </section>
    {:else if library}
      {#if library.books.length === 0}
        <section class="card empty">
          <div class="book-icon" aria-hidden="true">▥</div>
          <p class="eyebrow">Здесь появится ваша коллекция</p>
          <h2>Начните с одной важной книги</h2>
          <p>
            Bookshelf хранит книги и рабочее состояние на этом компьютере. Импорт PDF станет
            доступен на следующем шаге.
          </p>
        </section>
      {/if}

      <section class="card note-card">
        <div>
          <p class="eyebrow">Проверка рабочего состояния</p>
          <h2>Заметка пространства</h2>
          <p>Сохраните короткую пометку — она останется здесь после перезапуска приложения.</p>
        </div>
        <form onsubmit={saveNote}>
          <label for="workspace-note">Пометка</label>
          <div class="field-row">
            <input
              id="workspace-note"
              bind:value={note}
              oninput={() => (saved = false)}
              placeholder="Например, выбрать следующую книгу"
              maxlength="240"
            />
            <button class="secondary" disabled={saving}>{saving ? "Сохраняем…" : "Сохранить"}</button>
          </div>
          {#if saved}<p class="success" role="status">Сохранено локально</p>{/if}
          {#if error}<p class="inline-error" role="alert">{error}</p>{/if}
        </form>
      </section>
    {/if}
  </main>
</div>

<style>
  :global(*) { box-sizing: border-box; }
  :global(body) { margin: 0; min-width: 720px; background: #f4f3ef; color: #20211d; font-family: Inter, ui-sans-serif, system-ui, sans-serif; }
  :global(button), :global(input) { font: inherit; }
  .shell { min-height: 100vh; display: grid; grid-template-columns: 248px 1fr; }
  aside { background: #20251f; color: #f6f3e9; padding: 30px 22px; display: flex; flex-direction: column; }
  .brand { display: flex; align-items: center; gap: 10px; font: 650 18px Georgia, serif; letter-spacing: .02em; }
  .brand span { display: grid; place-items: center; width: 34px; height: 34px; border-radius: 9px; background: #c8d89a; color: #26301f; }
  nav { margin-top: 48px; display: grid; gap: 8px; }
  nav a, nav span { color: #c7cabf; text-decoration: none; padding: 11px 12px; border-radius: 8px; font-size: 14px; }
  nav .active { color: white; background: #343c32; }
  nav small { float: right; padding: 1px 7px; border-radius: 10px; background: #485044; }
  .local { margin-top: auto; color: #9da496; font-size: 12px; }
  .local i { display: inline-block; width: 7px; height: 7px; border-radius: 50%; background: #accb74; margin-right: 6px; }
  main { padding: 54px clamp(40px, 7vw, 92px); max-width: 1150px; width: 100%; }
  header { display: flex; justify-content: space-between; align-items: end; margin-bottom: 30px; }
  h1 { margin: 4px 0 0; font: 500 42px/1.1 Georgia, serif; }
  h2 { margin: 7px 0 10px; font: 500 25px/1.25 Georgia, serif; }
  p { line-height: 1.55; }
  .eyebrow { margin: 0; color: #68705f; font-size: 12px; font-weight: 700; letter-spacing: .1em; text-transform: uppercase; }
  button { border: 0; border-radius: 8px; padding: 11px 17px; font-weight: 650; }
  button:disabled { opacity: .48; cursor: not-allowed; }
  .primary { background: #526843; color: white; }
  .secondary { background: #2f392c; color: white; min-width: 116px; }
  .card { border: 1px solid #deddd6; border-radius: 14px; background: #fff; box-shadow: 0 5px 18px rgba(34, 38, 31, .04); }
  .empty { padding: 54px 40px; text-align: center; }
  .empty > p:last-child { max-width: 540px; margin: 0 auto; color: #686a63; }
  .book-icon { width: 58px; height: 58px; margin: 0 auto 22px; display: grid; place-items: center; border-radius: 16px; background: #edf1df; color: #61734c; font-size: 30px; }
  .note-card { display: grid; grid-template-columns: minmax(220px, .75fr) minmax(320px, 1fr); gap: 42px; padding: 30px 34px; margin-top: 20px; }
  .note-card p { color: #6c6e67; margin-bottom: 0; }
  form { align-self: center; }
  label { display: block; margin-bottom: 7px; font-size: 13px; font-weight: 700; }
  .field-row { display: flex; gap: 8px; }
  input { width: 100%; min-width: 0; padding: 11px 13px; border: 1px solid #cfd1c9; border-radius: 8px; color: #20211d; background: #fafaf8; outline: none; }
  input:focus { border-color: #657c53; box-shadow: 0 0 0 3px #dce5ce; }
  .success, .inline-error { margin: 8px 0 0; font-size: 13px; }
  .success { color: #537038 !important; }
  .inline-error, .error { color: #9b3731; }
  .status, .error { padding: 28px; }
  @media (max-width: 820px) { .note-card { grid-template-columns: 1fr; gap: 18px; } }
</style>
