# 04 — Доступные shared UI primitives

**Spec:** [Предсказуемая архитектура Bookshelf](../spec.md)

**What to build:** Экранные сценарии получают единый набор доступных select, checkbox, dialog, tooltip и icon-action primitives на Bits UI и Lucide, сохраняя визуальную систему Bookshelf.

**Blocked by:** 01 — Воспроизводимые локальные quality gates и toolchains.

**Status:** ready-for-agent

- [ ] Bits UI импортируется только внутри shared UI primitives, а экранные компоненты используют публичные wrappers.
- [ ] Select, checkbox, dialog и tooltip поддерживают ожидаемые keyboard, focus и accessible-name сценарии.
- [ ] Нативные input и textarea остаются в собственных типизированных wrappers.
- [ ] Lucide-иконки дополняют текст; icon-only primitive требует доступное имя и tooltip.
- [ ] Компонентные тесты проверяют внешнее доступное поведение, а не внутреннюю структуру Bits UI.
- [ ] Проходят все frontend-команды из `AGENTS.md`.
