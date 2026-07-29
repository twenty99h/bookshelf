# 06 — Стабильный IPC-контракт ошибок

**Spec:** [Предсказуемая архитектура Bookshelf](../spec.md)

**What to build:** Самостоятельный читатель продолжает видеть понятные прежние сообщения об ошибках, а frontend получает стабильные машинные коды независимо от внутренних Rust error chains.

**Blocked by:** 02 — Воспроизводимая генерация TypeScript contracts.

**Status:** ready-for-agent

- [ ] Внутренние domain/application/adapter errors представлены закрытыми `thiserror` enums на затронутых границах.
- [ ] IPC отображает ошибки в сериализуемый `{ code, message }` со стабильным конечным набором codes.
- [ ] Существующие видимые пользователю сообщения сохранены там, где поведение не меняется.
- [ ] Contract tests проверяют валидацию входа и отображение репрезентативных domain, persistence, filesystem и external-process ошибок.
- [ ] TypeScript bindings регенерированы, и изменились только ожидаемые contracts.
- [ ] Проходят все frontend- и Rust-команды из `AGENTS.md`.
