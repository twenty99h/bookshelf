<script lang="ts">
  import { onMount } from "svelte";
  import { FileArchive, Library, Menu, Sparkles } from "@lucide/svelte";
  import { Button, CheckboxField, SelectField, TextField } from "@/shared/ui";
  import type { LibraryState } from "@/shared/api";

  type SettingsSection = "interface" | "library" | "backups" | "ai";

  let {
    section = $bindable(),
    library,
    readerMode = $bindable(),
    readerImages = $bindable(),
    updateStatus,
    diagnosticStatus,
    onRestoreBackup,
    onExportArchive,
    onImportArchive,
    onExportDiagnostics,
    onCheckForUpdate,
    onPersistPreferences,
  }: {
    section: SettingsSection;
    library: LibraryState;
    readerMode: string;
    readerImages: boolean;
    updateStatus: string;
    diagnosticStatus: string;
    onRestoreBackup: () => Promise<string>;
    onExportArchive: (password: string) => Promise<string>;
    onImportArchive: (password: string) => Promise<string>;
    onExportDiagnostics: () => Promise<void>;
    onCheckForUpdate: () => Promise<void>;
    onPersistPreferences: () => Promise<void>;
  } = $props();

  let backupPassword = $state("");
  let backupStatus = $state("");
  let lastArchiveAt = $state<number | null>(null);
  const snapshotAt = $derived(Math.max(0, ...library.milestones.map((milestone) => milestone.occurredAt)) || null);
  const changesSinceArchive = $derived(
    library.milestones.filter((milestone) => !lastArchiveAt || milestone.occurredAt > lastArchiveAt).length,
  );
  const archiveIsOld = $derived(!lastArchiveAt || Date.now() / 1_000 - lastArchiveAt > 30 * 86_400);
  const shouldPromptExport = $derived(changesSinceArchive >= 5 || (archiveIsOld && changesSinceArchive > 0));

  onMount(() => {
    const stored = Number(localStorage.getItem("bookshelf-last-archive-at"));
    lastArchiveAt = stored > 0 ? stored : null;
  });

  function formatDate(timestamp: number | null) {
    return timestamp
      ? new Intl.DateTimeFormat("ru-RU", { dateStyle: "long", timeStyle: "short" }).format(new Date(timestamp * 1_000))
      : "ещё не создавалась";
  }

  async function restoreBackup() {
    backupStatus = "Восстановление…";
    backupStatus = await onRestoreBackup();
  }

  async function exportArchive() {
    backupStatus = "Экспорт…";
    backupStatus = await onExportArchive(backupPassword);
    if (backupStatus === "Переносимый архив сохранён") {
      lastArchiveAt = Math.floor(Date.now() / 1_000);
      localStorage.setItem("bookshelf-last-archive-at", String(lastArchiveAt));
    }
  }

  async function importArchive() {
    backupStatus = "Импорт…";
    backupStatus = await onImportArchive(backupPassword);
  }

  const navigation = [
    { id: "interface" as const, icon: Menu, label: "Интерфейс" },
    { id: "library" as const, icon: Library, label: "Библиотека" },
    { id: "backups" as const, icon: FileArchive, label: "Резервные копии" },
    { id: "ai" as const, icon: Sparkles, label: "ИИ" },
  ];
</script>

<div class="grid grid-cols-[220px_minmax(0,1fr)] gap-7">
  <nav aria-label="Разделы настроек" class="grid content-start gap-1 rounded-xl border border-white/8 bg-slate p-3">
    {#each navigation as item (item.label)}{@const Icon = item.icon}<button
        aria-current={section === item.id ? "page" : undefined}
        class="flex items-center gap-3 rounded-md px-3 py-3 text-left text-sm text-mist-dim aria-[current=page]:bg-iris/12 aria-[current=page]:text-mist"
        onclick={() => (section = item.id)}><Icon class="size-4" />{item.label}</button
      >{/each}
  </nav>
  <div class="grid gap-5">
    {#if section === "interface"}<section class="rounded-xl border border-white/8 bg-slate p-7">
        <p class="font-mono text-xs uppercase tracking-[.14em] text-iris">Интерфейс</p>
        <h2 class="mt-3 text-xl font-semibold">Чтение и рабочее пространство</h2>
        <div class="mt-6 grid gap-5">
          <SelectField
            label="Режим документа по умолчанию"
            value={readerMode}
            options={[
              { value: "muted", label: "Приглушённый светлый" },
              { value: "original", label: "Оригинальный" },
              { value: "dark", label: "Тёмный инвертированный" },
            ]}
            onValueChange={(value) => {
              readerMode = value;
              void onPersistPreferences();
            }}
          /><CheckboxField
            id="invert-images"
            label="Инвертировать изображения в тёмном режиме"
            bind:checked={readerImages}
            onCheckedChange={() => void onPersistPreferences()}
          />
        </div>
      </section>
    {:else if section === "library"}<section class="rounded-xl border border-white/8 bg-slate p-7">
        <p class="font-mono text-xs uppercase tracking-[.14em] text-iris">Библиотека</p>
        <h2 class="mt-3 text-xl font-semibold">Локальные книги и диагностика</h2>
        <p class="mt-3 text-sm leading-6 text-mist-dim">
          PDF, позиции чтения и авторские записи остаются на этом компьютере. Телеметрия и автоматическая отправка
          журнала отключены.
        </p>
        <div class="mt-6 rounded-lg border border-white/8 bg-night/30 p-5">
          <b>Диагностический журнал</b>
          <p class="mt-2 text-sm text-mist-dim">Хранятся только последние 100 локальных записей текущего запуска.</p>
          <Button onclick={onExportDiagnostics}>Экспортировать журнал</Button>{#if diagnosticStatus}<p
              class="mt-3 text-sm text-mist-dim"
              role="status"
            >
              {diagnosticStatus}
            </p>{/if}
        </div>
      </section>
    {:else if section === "backups"}<section class="rounded-xl border border-white/8 bg-slate p-7">
        <div class="flex items-start justify-between">
          <div>
            <p class="font-mono text-xs uppercase tracking-[.14em] text-amber">Резервные копии</p>
            <h2 class="mt-3 text-xl font-semibold">Локальное восстановление</h2>
          </div>
          <span class="font-mono text-xs text-success">Snapshot: {formatDate(snapshotAt)}</span>
        </div>
        {#if shouldPromptExport}<p
            class="mt-5 rounded-lg border border-amber/35 bg-amber/10 p-4 text-sm text-amber"
            role="status"
          >
            После последнего архива накопились существенные изменения или архив давно не создавался. Сохраните
            переносимую копию, когда будет удобно.
          </p>{/if}
        <div class="mt-6 grid grid-cols-2 gap-4">
          <div class="rounded-lg border border-white/8 bg-night/30 p-5">
            <b>Автоматический snapshot</b>
            <p class="mt-2 text-sm leading-6 text-mist-dim">Последняя внутренняя копия: {formatDate(snapshotAt)}.</p>
            <Button onclick={restoreBackup}>Восстановить последний</Button>
          </div>
          <div class="rounded-lg border border-white/8 bg-night/30 p-5">
            <b>Переносимый архив</b>
            <p class="mt-2 text-sm leading-6 text-mist-dim">
              Последний экспорт: {formatDate(lastArchiveAt)}. Архив защищён паролем.
            </p>
            <TextField id="backup-password" label="Пароль архива" type="password" bind:value={backupPassword} />
            <div class="flex gap-2">
              <Button disabled={!backupPassword} onclick={exportArchive}>Экспортировать</Button><Button
                disabled={!backupPassword}
                onclick={importArchive}>Импортировать</Button
              >
            </div>
          </div>
        </div>
        {#if backupStatus}<p class="mt-4 text-sm text-mist-dim" role="status">{backupStatus}</p>{/if}
      </section>
    {:else}<section class="rounded-xl border border-white/8 bg-slate p-7">
        <p class="font-mono text-xs uppercase tracking-[.14em] text-iris">ИИ</p>
        <h2 class="mt-3 text-xl font-semibold">Явные проверки Codex</h2>
        <p class="mt-3 max-w-2xl text-sm leading-6 text-mist-dim">
          Codex запускается только из детали идеи после просмотра минимального пакета. PDF целиком и другие записи
          автоматически не передаются.
        </p>
        <p class="mt-5 text-sm text-mist-dim">Вход выполняется локальным процессом Codex при первом явном запросе.</p>
      </section>{/if}
    <section class="rounded-xl border border-white/8 bg-slate p-7">
      <div class="flex items-center">
        <div>
          <b>Обновления Bookshelf</b>
          <p class="mt-1 text-sm text-mist-dim">Версия 0.1.0 · проверка выполняется только явно.</p>
        </div>
        <Button onclick={onCheckForUpdate}>Проверить обновления</Button>
      </div>
      {#if updateStatus}<p class="mt-3 text-sm text-mist-dim" role="status">{updateStatus}</p>{/if}
    </section>
  </div>
</div>
