# 12 — Учебные сессии и завершение изучения

**Spec:** [Предсказуемая архитектура Bookshelf](../spec.md)

**What to build:** Самостоятельный читатель может проводить учебные сессии и практические эксперименты, а затем явно завершить изучение книги с обязательной ретроспективой.

**Blocked by:** 11 — Восстановление знания.

**Status:** ready-for-agent

- [x] Статус учебной сессии и другие конечные состояния представлены закрытыми Rust/TypeScript types.
- [x] Domain rules защищают допустимые переходы, ретроспективу книги и продолжение открытых практических экспериментов после завершения изучения.
- [x] Application-сценарии используют repository/clock/id seams и не зависят от Tauri runtime.
- [x] Page UI хранит незавершённые формы локально и использует shared controls без изменения текстов и визуальной системы.
- [x] Page-level tests покрывают сессию, эксперимент и завершение изучения; Rust tests покрывают инварианты и persistence round trip.
- [x] Ticket сохраняет связь с [исходным Rust finding 06](../../bookshelf-code-review/issues/06-rust-domain-and-persistence-boundaries.md), не изменяя его.
- [x] Проходят все frontend- и Rust-команды из `AGENTS.md`; generated bindings имеют только ожидаемый diff.
