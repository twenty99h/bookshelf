# 04 — Review frontend boundaries and styling

Type: task
Status: resolved

## Review findings

- `IdeaWorkbench` combines ideas, recall, experiments, transfer, organization, and AI review.
- New styles use component CSS rather than Tailwind.
- Review kinds are primitive strings with repeated conditional cascades.
- IPC types are manually duplicated rather than checked/generated from the Rust seam.

## Acceptance

- [x] Extract recall and Codex review into focused scenario components with public APIs.
- [x] Express newly changed presentation with Tailwind utilities.
- [x] Centralize typed review-kind labels/questions.
- [x] Add a binding contract check so Rust/TypeScript IPC drift fails validation.

## Answer

`RecallPanel` and `CodexReviewPanel` own their focused flows, new presentation uses Tailwind, and typed review copy replaces conditional strings. `ts-rs` now generates the frontend state/action contracts during Rust tests; the frontend imports those files directly.
