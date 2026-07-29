# 01 — Review reading, study debt, and recall

Type: task
Status: resolved

## Review findings

- Nested PDF outline items are dropped instead of being available to navigation.
- Debt change is measured from session planning, not the start of the performed session.
- Deferred reviews have no visible action, and due recalls cannot be inspected, moved, or started early.

## Acceptance

- [x] PDF outline traversal preserves nested items and parent relationships.
- [x] A reader explicitly starts a session before debt delta measurement begins.
- [x] Pending reviews and scheduled recalls expose direct user actions.

## Answer

Nested outlines are flattened with stable parent IDs, session start captures the debt baseline, and the UI exposes start/reschedule/reopen actions. Domain persistence and scheduling tests cover the changed behavior.
