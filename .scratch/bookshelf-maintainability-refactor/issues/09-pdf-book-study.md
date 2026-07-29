# 09 — Изучение книги через PDF

**Spec:** [Предсказуемая архитектура Bookshelf](../spec.md)

**What to build:** Самостоятельный читатель может импортировать PDF, читать его, сохранить источник идеи из выделенного фрагмента и восстановить позицию после перезапуска через новые архитектурные границы.

**Blocked by:** 08 — Жизненный цикл черновой заметки.

**Status:** ready-for-agent

- [ ] Reading rules и application-сценарии отделены от PDF storage, SQLite и IPC adapters.
- [ ] Существующие незакоммиченные изменения PDF asset scope, viewer и локальных PDF.js assets сохранены и интегрированы без отката.
- [ ] Прямое управление DOM со стороны PDF.js ограничено отдельным Svelte integration seam и корректно очищается.
- [ ] Динамические списки чтения используют устойчивые keys; прикладной UI использует Tailwind и shared UI primitives.
- [ ] Page-level тест проверяет импорт, открытие, сохранение источника и восстановление позиции через fake command adapter.
- [ ] Rust integration tests используют реальные временные PDF/filesystem/SQLite resources; PDF corpus фиксирует известные сложные документы.
- [ ] Ticket сохраняет связь с [исходным Rust finding 06](../../bookshelf-code-review/issues/06-rust-domain-and-persistence-boundaries.md), не изменяя его.
- [ ] Проходят все frontend- и Rust-команды из `AGENTS.md`; generated bindings имеют только ожидаемый diff.
