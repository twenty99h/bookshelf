# 02 — Воспроизводимая генерация TypeScript contracts

**Spec:** [Предсказуемая архитектура Bookshelf](../spec.md)

**What to build:** Разработчик может одной документированной командой получить TypeScript bindings из канонических Rust snapshot/command types и проверить, что сохранённые bindings соответствуют исходным контрактам.

**Blocked by:** 01 — Воспроизводимые локальные quality gates и toolchains.

**Status:** ready-for-agent

- [ ] Канонические snapshot/command types генерируют bindings через `ts-rs` одной воспроизводимой командой.
- [ ] Проверка завершается ошибкой, если повторная генерация создаёт неожиданный diff.
- [ ] Generated bindings не форматируются и не редактируются вручную.
- [ ] Повторная генерация на неизменном дереве не создаёт diff.
- [ ] Проходят все frontend- и Rust-команды из `AGENTS.md`.
