<script lang="ts">
  import { FileArchive, Library, Menu, Sparkles } from "@lucide/svelte";
  import { Button, CheckboxField, SelectField, TextField } from "@/shared/ui";

  type SettingsSection = "interface" | "library" | "backups" | "ai";

  let {
    section = $bindable(),
    readerMode = $bindable(),
    readerImages = $bindable(),
    backupPassword = $bindable(),
    backupStatus,
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
    readerMode: string;
    readerImages: boolean;
    backupPassword: string;
    backupStatus: string;
    updateStatus: string;
    diagnosticStatus: string;
    onRestoreBackup: () => Promise<void>;
    onExportArchive: () => Promise<void>;
    onImportArchive: () => Promise<void>;
    onExportDiagnostics: () => Promise<void>;
    onCheckForUpdate: () => Promise<void>;
    onPersistPreferences: () => Promise<void>;
  } = $props();

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
          <span class="font-mono text-xs text-success">Snapshot сегодня, 18:40</span>
        </div>
        <div class="mt-6 grid grid-cols-2 gap-4">
          <div class="rounded-lg border border-white/8 bg-night/30 p-5">
            <b>Автоматический snapshot</b>
            <p class="mt-2 text-sm leading-6 text-mist-dim">Последняя внутренняя копия: 30 июля 2026, 18:40.</p>
            <Button onclick={onRestoreBackup}>Восстановить последний</Button>
          </div>
          <div class="rounded-lg border border-white/8 bg-night/30 p-5">
            <b>Переносимый архив</b>
            <p class="mt-2 text-sm leading-6 text-mist-dim">Последний экспорт: 12 июля 2026. Архив защищён паролем.</p>
            <TextField id="backup-password" label="Пароль архива" type="password" bind:value={backupPassword} />
            <div class="flex gap-2">
              <Button disabled={!backupPassword} onclick={onExportArchive}>Экспортировать</Button><Button
                disabled={!backupPassword}
                onclick={onImportArchive}>Импортировать</Button
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
