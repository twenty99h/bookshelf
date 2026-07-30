# 16 — Завершение Tailwind/Bits UI migration

**Spec:** [Предсказуемая архитектура Bookshelf](../spec.md)

**What to build:** Самостоятельный читатель получает тот же визуальный интерфейс с последовательным keyboard/focus поведением, а сопровождающий — единый styling contract без оставшихся прикладных CSS-исключений.

**Blocked by:** 09 — Изучение книги через PDF; 10 — Работа с идеями книги; 11 — Восстановление знания; 12 — Учебные сессии и завершение изучения; 13 — Поиск и экспорт материалов; 14 — Проверка идеи через Codex; 15 — Архив, снимки и настройки личной библиотеки.

**Status:** ready-for-agent

- [x] Весь оставшийся прикладной UI использует Tailwind utilities без `<style>`, `@apply` и новых семантических CSS-классов.
- [x] Глобальные стили содержат только Tailwind import, theme tokens, reset и reduced-motion baseline.
- [x] Единственное локальное CSS-исключение содержит только необходимые PDF.js vendor/display-layer rules без Bookshelf selectors.
- [x] Сложные controls используют shared Bits UI wrappers; повторяющееся оформление выражено компонентами, а не новыми CSS classes.
- [x] Visual system, layout и пользовательские тексты не изменены; keyboard, focus, accessible names и reduced-motion защищены тестами.
- [x] Проходят все frontend-команды из `AGENTS.md`.
