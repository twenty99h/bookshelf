# 06 — Review Rust domain and persistence boundaries

Type: task
Status: ready-for-agent

## Review findings

- `library.rs` combines pure domain state/rules, application orchestration, SQLite persistence, PDF storage, encrypted archives, and Markdown export.
- This conflicts with the documented `domain`, `application`, `adapters/sqlite`, `adapters/library`, and `adapters/archive` ownership seams.

## Acceptance

- [ ] Move pure state, actions, validation, and debt rules into `domain` without filesystem, SQLite, or Tauri dependencies.
- [ ] Put use-case orchestration behind `application` APIs.
- [ ] Isolate SQLite/snapshots, PDF storage, and encrypted archive behavior in their documented adapters.
- [ ] Preserve the existing persistence, archive, search, and domain regression tests through the move.

## Answer

Second-pass standards review recorded this as structural debt. It is intentionally not marked resolved by the feature implementation commit because moving persistence boundaries safely is a separate refactor.
