# 15 — Архив, снимки и настройки личной библиотеки

**Spec:** [Предсказуемая архитектура Bookshelf](../spec.md)

**What to build:** Самостоятельный читатель может экспортировать и импортировать переносимый архив, восстановить последний согласованный снимок и управлять связанными настройками без смешивания domain, archive, SQLite, filesystem и IPC responsibilities.

**Blocked by:** 13 — Поиск и экспорт материалов; 14 — Проверка идеи через Codex.

**Status:** ready-for-agent

- [ ] Archive и snapshot application APIs оркестрируют отдельные SQLite/filesystem/archive adapters через небольшие интерфейсы.
- [ ] Успешный импорт или restore возвращает полный атомарный `LibraryState` согласно ADR-0006.
- [ ] Archive round-trip на реальных временных ресурсах покрывает верный пароль, неверный пароль, повреждение и прерванную запись.
- [ ] Codex login state, logs и transient AI state отсутствуют в архиве.
- [ ] Development format может измениться несовместимо, но новый актуальный format закреплён round-trip tests без legacy aliases.
- [ ] Page-level tests через fake command adapter покрывают export, import, restore, progress/error и связанные настройки.
- [ ] Ticket сохраняет связь с [исходным Rust finding 06](../../bookshelf-code-review/issues/06-rust-domain-and-persistence-boundaries.md), не изменяя его.
- [ ] Проходят все frontend- и Rust-команды из `AGENTS.md`; generated bindings имеют только ожидаемый diff.
