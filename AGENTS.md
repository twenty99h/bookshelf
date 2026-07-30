# Repository Guidelines

## Agent-Specific Guidelines

- Be concise.
- Follow Feature-Sliced Design for frontend architecture. Use the Feature-Sliced Design skill when deciding module placement, boundaries, or imports.
- Use Tailwind CSS for styling and Bits UI for reusable UI primitives.
- Consult Bits UI documentation at `https://bits-ui.com/docs/components/{component_name}/llms.txt` before implementing or modifying a Bits UI component.

## Project Structure & Module Organization

Bookshelf is a Svelte 5/SvelteKit desktop app backed by Tauri 2 and Rust. In `src/`, routes live in `routes`, page composition in `pages`, capabilities in `features`, reusable code in `shared`, and global styles in `app`. Import slices through public `index.ts` APIs. Tauri commands, persistence, and adapters live under `src-tauri/src`. Put static files in `static` and project guidance in `docs`.

Keep tests near the behavior they cover, using names such as `library-page.test.ts`. Treat `src/shared/api/generated` as generated output, not hand-written code.

## Build, Test, and Development Commands

- `pnpm install` installs dependencies (requires Node >=24.18 and <25, with pnpm 11.18.0).
- `pnpm dev` starts the Vite frontend; `pnpm tauri dev` runs the desktop application.
- `pnpm format:check && pnpm lint && pnpm check && pnpm test && pnpm build` is the required frontend acceptance suite.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` verifies Rust formatting.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` rejects Rust warnings.
- `cargo test --manifest-path src-tauri/Cargo.toml` runs backend tests.

## Coding Style & Naming Conventions

Use Prettier, ESLint, TypeScript strict checks, and `rustfmt`. Name Svelte components in PascalCase (`RecallPanel.svelte`), TypeScript values in camelCase, and Rust modules/functions in snake_case. Keep domain logic out of route files.

## Testing Guidelines

Frontend tests use Vitest, Testing Library, and jsdom. Use behavior-oriented descriptions, accessible queries, and explicit Tauri mocks. No coverage threshold is enforced; add focused tests for new behavior and regressions. When Rust contract types change, regenerate bindings with `ts-rs` and confirm only expected files changed in `src/shared/api/generated`.

## Commit & Pull Request Guidelines

Recent history follows concise Conventional Commit subjects: `feat:`, `fix:`, `refactor:`, `docs:`, and `chore:`. Keep commits scoped and imperative. Pull requests should explain the user-visible change, link the relevant issue, list verification commands, and include screenshots for UI changes. Do not report completion while applicable acceptance checks fail.

## Issue Tracker

Issues are tracked as local Markdown files under `.scratch/`. See `docs/agents/issue-tracker.md`.

## Triage Labels

Triage uses the five default canonical label strings. See `docs/agents/triage-labels.md`.

## Domain Docs

Domain documentation uses the single-context layout. See `docs/agents/domain.md`.
