# 01 — Воспроизводимые локальные quality gates и toolchains

**Spec:** [Предсказуемая архитектура Bookshelf](../spec.md)

**What to build:** Разработчик получает один воспроизводимый локальный набор форматирования, lint, проверки типов, тестов и сборки, работающий на зафиксированных версиях Node, pnpm и Rust без попутного массового обновления зависимостей.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] В проекте доступны команды `pnpm format:check`, `pnpm lint`, `pnpm check`, `pnpm test` и `pnpm build` с локальными Prettier и ESLint configs.
- [ ] Существующий frontend один раз механически отформатирован отдельно от поведенческих изменений.
- [ ] Поддерживаемые версии Node, pnpm и Rust зафиксированы воспроизводимым способом; lockfiles не обновлены массово без необходимости.
- [ ] Все пять frontend-команд проходят локально.
- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` и `cargo test --manifest-path src-tauri/Cargo.toml` проходят локально.
