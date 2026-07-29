# 02 — Review the Codex boundary

Type: task
Status: resolved

## Review findings

- Codex IPC input is not validated before the external request.
- A PATH fallback permits an independently updated Codex binary.
- Generated App Server schema is unused, bootstrap logic is duplicated, and closed protocol/domain values are strings.
- Backend behavior remains in flat modules instead of the documented domain/application/adapter/IPC seams.

## Acceptance

- [x] Validate idea, request id, request kind, and exact package before launching Codex.
- [x] Production resolves only the bundled sidecar; an explicit development override remains possible.
- [x] Pin and compatibility-test the stable JSONL messages used by Bookshelf.
- [x] Extract reusable Codex connection setup and use closed Rust enums for request/decision values.
- [x] Separate adapter and IPC ownership enough that external process details do not leak into commands.

## Answer

The Codex process/protocol lives in `adapters/codex`, commands validate domain-owned review packages first, and production only accepts the paired sidecar. Protocol fixtures and release schema assertions cover the stable App Server surface.
