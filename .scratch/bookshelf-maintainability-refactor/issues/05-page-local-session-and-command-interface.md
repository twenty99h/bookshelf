# 05 — Page-local session и интерфейс команд

**Spec:** [Предсказуемая архитектура Bookshelf](../spec.md)

**What to build:** Страница личной библиотеки сохраняет текущие сценарии, но получает состояние через явно внедряемый интерфейс предметных команд; тесты могут управлять всей страницей без низкоуровневого Tauri transport.

**Blocked by:** 02 — Воспроизводимая генерация TypeScript contracts; 03 — Строгая проверка TypeScript contracts.

**Status:** ready-for-agent

- [x] Page-local session единолично владеет атомарным `LibraryState`, типизированной навигацией и общими loading/error-состояниями.
- [x] Изменяющие команды выполняются в определённом порядке, а поздний устаревший ответ не заменяет более новый снимок.
- [x] Production Tauri adapter и явно переданный in-memory fake реализуют один узкий интерфейс предметных команд.
- [x] Page-level тесты через Testing Library проверяют loading, ошибку, навигацию, очередь действий и атомарную замену снимка без mock Tauri `invoke`.
- [x] Ручная нормализация с fallback-полями для сгенерированного `LibraryState` удалена.
- [x] Глобальный store, TanStack Query и универсальный component dispatch не добавлены.
- [x] Проходят все frontend-команды из `AGENTS.md`; generated bindings не имеют неожиданного diff.
