# 21 — Production hardening и финальный визуальный контракт

**What to build:** Подтвердить, что целевой UX Bookshelf воспроизводимо работает как доступное Windows-first desktop-приложение, а не только как browser harness, и закрепить согласованный визуальный контракт в CI.

**Blocked by:** 02–20 — все пользовательские срезы UX-переработки.

**Status:** ready-for-agent

- [ ] Визуальная матрица содержит и стабильно проходит все 12 baselines: `dashboard-active`, `library-list`, `book-overview`, `reader-clean`, `reader-note`, `reader-outline`, `drafts-review`, `idea-detail`, `practice-active`, `book-completion`, `settings-backup`, `first-run`.
- [ ] Основные снимки закреплены при 1920×1080, функциональный suite проходит при 1280×720, а систематический недетерминизм устранён до выбора pixel tolerance.
- [ ] Ключевые страницы проходят автоматическую accessibility-проверку и ручной аудит focus order, announcements, contrast, color-only сигналов и reduced motion.
- [ ] Интерфейс остаётся рабочим при поддерживаемом Windows scaling и временном minimum window без горизонтальной прокрутки основного содержимого.
- [ ] Нативный smoke suite проверяет системный импорт PDF, выдачу локального URL, открытие reader и восстановление позиции после перезапуска и не повторяет полную visual matrix.
- [ ] CI запускает frontend formatting, lint, type/Svelte checks, Vitest, Playwright и production build, а Rust — rustfmt, Clippy без warnings и Cargo tests.
- [ ] Visual mismatch публикует expected, actual и diff; baseline обновляется только явной локальной командой с объяснением пользовательского изменения.
- [ ] Финальная проверка подтверждает отсутствие реального Codex в E2E, browser adapter в production bundle, удалённых шрифтов и произвольных PDF-ready timeouts.
