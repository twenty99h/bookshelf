# 05 — Page-local session и интерфейс команд

**Spec:** [Предсказуемая архитектура Bookshelf](../spec.md)

**What to build:** Страница личной библиотеки сохраняет текущие сценарии, но получает состояние через явно внедряемый интерфейс предметных команд; тесты могут управлять всей страницей без низкоуровневого Tauri transport.

**Blocked by:** 02 — Воспроизводимая генерация TypeScript contracts; 03 — Строгая проверка TypeScript contracts.

**Status:** ready-for-agent

- [ ] Page-local session единолично владеет атомарным `LibraryState`, типизированной навигацией и общими loading/error-состояниями.
- [ ] Изменяющие команды выполняются в определённом порядке, а поздний устаревший ответ не заменяет более новый снимок.
- [ ] Production Tauri adapter и явно переданный in-memory fake реализуют один узкий интерфейс предметных команд.
- [ ] Page-level тесты через Testing Library проверяют loading, ошибку, навигацию, очередь действий и атомарную замену снимка без mock Tauri `invoke`.
- [ ] Ручная нормализация с fallback-полями для сгенерированного `LibraryState` удалена.
- [ ] Глобальный store, TanStack Query и универсальный component dispatch не добавлены.
- [ ] Проходят все frontend-команды из `AGENTS.md`; generated bindings не имеют неожиданного diff.
