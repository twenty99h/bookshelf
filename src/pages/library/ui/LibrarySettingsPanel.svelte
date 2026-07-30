<script lang="ts">
  import { onMount } from "svelte";
  import { Button, NumberField, StatusMessage, TextField } from "@/shared/ui";
  import { commandErrorMessage, type LibraryAction, type LibraryState } from "@/shared/api";
  import type { LibraryCommands } from "../model/library-commands";

  let {
    library,
    commands,
    execute,
    onLibrary,
    onSnapshotRequest,
    onFeedback,
    onError,
  }: {
    library: LibraryState;
    commands: LibraryCommands;
    execute: (action: LibraryAction, success: string) => Promise<boolean>;
    onLibrary: (state: LibraryState, order: number) => void;
    onSnapshotRequest: () => number;
    onFeedback: (message: string) => void;
    onError: (message: string) => void;
  } = $props();

  let note = $state("");
  let saved = $state(false);
  let archivePassword = $state("");
  let busy = $state(false);
  let codexLoginUrl = $state("");
  let codexLoginCode = $state("");
  let codexLoginRunning = $state(false);

  onMount(() => {
    note = library.workspaceNote;
    let stop: (() => void) | undefined;
    void commands
      .onCodexLogin((event) => {
        if (event.kind !== "deviceCode") return;
        const [loginUrl = "", loginCode = ""] = event.text.split("\n", 2);
        codexLoginUrl = loginUrl;
        codexLoginCode = loginCode;
      })
      .then((unlisten) => (stop = unlisten));
    return () => stop?.();
  });

  async function saveNote(event: SubmitEvent) {
    event.preventDefault();
    saved = await execute({ kind: "saveWorkspaceNote", note }, "Сохранено локально");
  }

  async function exportArchive() {
    busy = true;
    onError("");
    try {
      if (await commands.exportArchive(archivePassword)) onFeedback("Зашифрованный архив сохранён");
    } catch (cause) {
      onError(commandErrorMessage(cause));
    } finally {
      busy = false;
    }
  }

  async function importArchive() {
    busy = true;
    onError("");
    const order = onSnapshotRequest();
    try {
      const snapshot = await commands.importArchive(archivePassword);
      if (!snapshot) return;
      onLibrary(snapshot, order);
      note = snapshot.workspaceNote;
      onFeedback("Личная библиотека восстановлена; вход в Codex потребуется выполнить заново");
    } catch (cause) {
      onError(commandErrorMessage(cause));
    } finally {
      busy = false;
    }
  }

  async function restoreSnapshot() {
    const order = onSnapshotRequest();
    try {
      const snapshot = await commands.restoreLatestSnapshot();
      onLibrary(snapshot, order);
      note = snapshot.workspaceNote;
      onFeedback("Последний снимок восстановлен");
    } catch (cause) {
      onError(commandErrorMessage(cause));
    }
  }

  async function installUpdate() {
    try {
      const installed = await commands.installSignedUpdate();
      onFeedback(installed ? "Подписанное обновление установлено" : "У вас актуальная версия");
    } catch (cause) {
      onError(commandErrorMessage(cause));
    }
  }

  async function loginCodex() {
    codexLoginRunning = true;
    onError("");
    codexLoginUrl = "";
    codexLoginCode = "";
    try {
      await commands.startCodexLogin();
      onFeedback("Вход в Codex завершён");
    } catch (cause) {
      onError(commandErrorMessage(cause));
    } finally {
      codexLoginRunning = false;
    }
  }
</script>

<section
  class="mb-6 grid grid-cols-[.8fr_1.2fr] gap-[34px] rounded-xl border border-rule bg-paper-raised p-6 shadow-paper max-[640px]:grid-cols-1 [&_p]:leading-[1.55] [&_p]:text-ink-muted"
  aria-label="Настройки"
>
  <div>
    <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">Настройки</p>
    <h2>Личное напоминание</h2>
    <p>Короткая запись для себя — например, с чего начать в следующий раз.</p>
  </div>
  <form onsubmit={saveNote}>
    <TextField
      id="workspace-note"
      label="Личное напоминание"
      bind:value={note}
      placeholder="Например, вернуться к главе 2"
      maxlength={240}
      disabled={busy}
    />
    <div class="mt-2.5 flex flex-wrap items-center gap-[9px]">
      <Button type="submit" disabled={busy}>Сохранить</Button>
      {#if saved}<StatusMessage tone="success">Сохранено локально</StatusMessage>{/if}
    </div>
  </form>
  <div>
    <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">
      Перенос и восстановление
    </p>
    <h2>Зашифрованный архив</h2>
    <p>
      Архив содержит рабочее состояние и PDF. Забытый пароль восстановить невозможно. Данные входа Codex не переносятся.
    </p>
  </div>
  <div>
    <TextField
      id="archive-password"
      label="Пароль архива"
      bind:value={archivePassword}
      placeholder="Не менее 8 символов"
      disabled={busy}
      type="password"
    />
    <div class="mt-2.5 flex flex-wrap items-center gap-[9px]">
      <Button onclick={exportArchive} disabled={busy || archivePassword.length < 8}>Экспортировать</Button>
      <Button onclick={importArchive} disabled={busy || archivePassword.length < 8}>Импортировать</Button>
      <Button onclick={restoreSnapshot}>Восстановить снимок</Button>
    </div>
  </div>
  <div>
    <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">Обновления</p>
    <h2>Совместимая версия Bookshelf</h2>
    <p>
      Устанавливаются только обновления с проверяемой подписью. При ошибке текущая версия и личная библиотека остаются
      без изменений.
    </p>
  </div>
  <div class="mt-2.5 flex flex-wrap items-center gap-[9px]">
    <Button onclick={installUpdate}>Проверить обновления</Button>
  </div>
  <div>
    <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">
      Ненавязчивое напоминание
    </p>
    <h2>Долг изучения</h2>
    <p>Одно системное уведомление появится только вне чтения, если объём долга не менялся выбранное число дней.</p>
  </div>
  <div class="mt-2.5 flex flex-wrap items-center gap-[9px]">
    <NumberField
      id="debt-days"
      label="Период без изменений"
      ariaLabel="Дней без изменения долга"
      min={1}
      max={90}
      value={library.debtReminderDays || 7}
      onChange={(days) => execute({ kind: "setDebtReminder", days }, "Период напоминания сохранён")}
    />
  </div>
  <div>
    <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">Необязательная проверка</p>
    <h2>Вход в Codex</h2>
    <p>Codex хранит вход в отдельном каталоге. Bookshelf не читает OAuth-токены и не переносит их в архив.</p>
  </div>
  <div class="mt-2.5 flex flex-wrap items-center gap-[9px]">
    <Button disabled={codexLoginRunning} onclick={loginCodex}
      >{codexLoginRunning ? "Ожидаем вход…" : "Войти через ChatGPT"}</Button
    >
    {#if codexLoginUrl}<Button onclick={() => commands.openExternalUrl(codexLoginUrl)}>Открыть страницу входа</Button
      ><strong aria-live="polite">Код: {codexLoginCode}</strong>{/if}
  </div>
</section>
