# 08 — Жизненный цикл черновой заметки

**Spec:** [Предсказуемая архитектура Bookshelf](../spec.md)

**What to build:** Самостоятельный читатель может сохранить черновую заметку, превратить её в идею книги, присоединить к существующей идее, экспортировать или удалить через первый полный срез новой архитектуры.

**Blocked by:** 04 — Доступные shared UI primitives; 05 — Page-local session и интерфейс команд; 06 — Стабильный IPC-контракт ошибок.

**Status:** ready-for-agent

- [x] Правила черновой заметки находятся в чистом domain-коде, а сценарии выполняются application API без Tauri, SQLite и filesystem dependencies.
- [x] Небольшой `LibraryRepository`, production SQLite adapter, in-memory test adapter, `Clock` и `IdGenerator` поддерживают этот срез.
- [x] Сохранение черновой заметки проходит через `execute_library_action`; дублирующая изменяющая IPC-команда удалена вместе с frontend caller.
- [x] Форма владеет незавершённым локальным состоянием и передаёт узкие предметные callbacks.
- [x] Page-level тесты с fake command adapter покрывают все пять исходов жизненного цикла и ошибки.
- [x] Domain/application tests покрывают переходы и детерминированные ids/time; export проверен на временном файле.
- [x] Ticket сохраняет связь с [исходным Rust finding 06](../../bookshelf-code-review/issues/06-rust-domain-and-persistence-boundaries.md), не изменяя его.
- [x] Проходят все frontend- и Rust-команды из `AGENTS.md`; generated bindings имеют только ожидаемый diff.
