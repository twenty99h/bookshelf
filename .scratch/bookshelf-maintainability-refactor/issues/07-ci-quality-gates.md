# 07 — CI для обязательных quality gates

**Spec:** [Предсказуемая архитектура Bookshelf](../spec.md)

**What to build:** Сопровождающий проекта получает автоматическую проверку того же набора локальных quality gates для каждого изменения и production Tauri build на основной desktop-платформе.

**Blocked by:** 01 — Воспроизводимые локальные quality gates и toolchains; 02 — Воспроизводимая генерация TypeScript contracts.

**Status:** ready-for-agent

- [x] Linux CI запускает frontend format/lint/check/test/build, Rust fmt/clippy/test и проверку generated bindings.
- [x] Windows CI запускает Rust-проверки и production Tauri build.
- [x] CI использует зафиксированные Node, pnpm и Rust toolchains с корректным dependency caching.
- [x] Трёхплатформенная release-candidate матрица не добавляется в быстрый обязательный workflow.
- [x] Локально проходят все frontend- и Rust-команды из `AGENTS.md`; workflow syntax проверен доступным локальным способом.
