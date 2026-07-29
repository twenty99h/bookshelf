# 10 — Работа с идеями книги

**Spec:** [Предсказуемая архитектура Bookshelf](../spec.md)

**What to build:** Самостоятельный читатель может создавать и уточнять идеи книги, выбирать назначения идеи и подтверждать связи идей, сохраняя обязательный источник и авторский контроль.

**Blocked by:** 09 — Изучение книги через PDF.

**Status:** ready-for-agent

- [ ] Идеи книги, версии, назначения и связи имеют чистые domain rules и закрытые Rust/TypeScript types.
- [ ] Application-сценарии используют repository/clock/id seams и возвращают полный атомарный снимок.
- [ ] UI рабочей области идеи разбит на связные локальные компоненты с локальным состоянием форм и узкими callbacks.
- [ ] Обычный mutable `Map` не используется как реактивный источник данных; списки имеют устойчивые keys.
- [ ] Подходящие select, checkbox, dialog и tooltip используют shared UI primitives с keyboard/focus tests.
- [ ] Page-level tests покрывают создание, изменение, назначение и явное подтверждение связи; Rust tests покрывают инварианты и persistence round trip.
- [ ] Ticket сохраняет связь с [исходным Rust finding 06](../../bookshelf-code-review/issues/06-rust-domain-and-persistence-boundaries.md), не изменяя его.
- [ ] Проходят все frontend- и Rust-команды из `AGENTS.md`; generated bindings имеют только ожидаемый diff.
