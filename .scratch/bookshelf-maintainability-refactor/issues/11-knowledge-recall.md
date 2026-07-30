# 11 — Восстановление знания

**Spec:** [Предсказуемая архитектура Bookshelf](../spec.md)

**What to build:** Самостоятельный читатель может пройти восстановление знания и сохранить каноническую самооценку, не встречая устаревших или произвольных строковых состояний.

**Blocked by:** 10 — Работа с идеями книги.

**Status:** ready-for-agent

- [x] Recall rules и application-сценарии отделены от persistence, IPC и UI.
- [x] Самооценка восстановления представлена закрытым enum/union и сериализует «не восстановил» только как `notRecalled`.
- [x] `missed` отсутствует в Rust, TypeScript, generated bindings и новом сериализованном формате без legacy alias.
- [x] UI восстановления использует локальное состояние формы, предметные callbacks и доступные shared controls.
- [x] Page-level tests покрывают все варианты самооценки и ошибку; domain/application tests покрывают допустимые переходы и атомарное сохранение.
- [x] Изменившиеся contracts воспроизводимо регенерированы, и diff ограничен ожидаемыми bindings.
- [x] Ticket сохраняет связь с [исходным Rust finding 06](../../bookshelf-code-review/issues/06-rust-domain-and-persistence-boundaries.md), не изменяя его.
- [x] Проходят все frontend- и Rust-команды из `AGENTS.md`.
