# 13 — Поиск и экспорт материалов

**Spec:** [Предсказуемая архитектура Bookshelf](../spec.md)

**What to build:** Самостоятельный читатель может искать книги, идеи, темы знаний и материалы, а затем экспортировать нужный материал без смешивания application orchestration с FTS5 и filesystem behavior.

**Blocked by:** 12 — Учебные сессии и завершение изучения.

**Status:** ready-for-agent

- [ ] Search application API возвращает типизированные результаты и не раскрывает SQL или универсальный query interface frontend-коду.
- [ ] FTS5 indexing/querying находится в SQLite adapter и проверяется на временной реальной базе.
- [ ] Export orchestration отделён от filesystem adapter и сохраняет существующий пользовательский результат.
- [ ] Page-level tests через fake command adapter проверяют поиск, пустой результат, ошибку и экспорт выбранного материала.
- [ ] Rust integration tests покрывают индексирование значимых полей, типы результатов и временный export resource.
- [ ] Ticket сохраняет связь с [исходным Rust finding 06](../../bookshelf-code-review/issues/06-rust-domain-and-persistence-boundaries.md), не изменяя его.
- [ ] Проходят все frontend- и Rust-команды из `AGENTS.md`; generated bindings имеют только ожидаемый diff.
