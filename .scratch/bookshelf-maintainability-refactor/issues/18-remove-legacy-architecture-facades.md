# 18 — Удаление прежних архитектурных фасадов

**Spec:** [Предсказуемая архитектура Bookshelf](../spec.md)

**What to build:** Сопровождающий завершает expand–contract refactor: все пользовательские сценарии продолжают работать, но прежние god-module responsibilities, дублирующие команды и временные compatibility seams больше не существуют.

**Blocked by:** 08 — Жизненный цикл черновой заметки; 09 — Изучение книги через PDF; 10 — Работа с идеями книги; 11 — Восстановление знания; 12 — Учебные сессии и завершение изучения; 13 — Поиск и экспорт материалов; 14 — Проверка идеи через Codex; 15 — Архив, снимки и настройки личной библиотеки; 16 — Завершение Tailwind/Bits UI migration; 17 — Проверка атомарного снимка на большой библиотеке.

**Status:** ready-for-agent

- [x] Rust facade содержит только wiring и небольшие IPC entry points; domain, application и adapters имеют связные private-by-default responsibilities по предметным способностям.
- [x] Прежний смешанный library implementation, дублирующая команда сохранения заметки и временные compatibility exports удалены после подтверждения отсутствия callers.
- [x] Страница личной библиотеки и рабочая область идеи являются композиционными модулями с узкими props/callbacks, а не владельцами несвязанных сценариев.
- [x] Нет ручной нормализации generated `LibraryState`, низкоуровневых Tauri mocks, обычного mutable `Map` как реактивного источника или неустойчивых list keys.
- [x] Все существующие пользовательские сценарии защищены на page command seam и Rust application seam; adapters сохраняют integration coverage на реальных временных ресурсах.
- [x] Финальная архитектура соответствует ADR-0006 и [исходному Rust finding 06](../../bookshelf-code-review/issues/06-rust-domain-and-persistence-boundaries.md); finding не изменён и не закрыт этим ticket.
- [x] Проходят все frontend- и Rust-команды из `AGENTS.md`; повторная генерация bindings не создаёт неожиданного diff.
