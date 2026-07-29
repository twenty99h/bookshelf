## Agent skills

- Be concise.
- Prefer FSD in frontend code. Use Feature-Sliced Design Skill if you have no clue how to implement feature.
- Use Tailwind CSS for styling and Bits UI for components.
- How to access Bits UI components documentation. Go to bits-ui.com/docs/components/{component_name}/llms.txt.

### Acceptance checks

Frontend changes are accepted only after these local commands pass:

- `pnpm format:check`
- `pnpm lint`
- `pnpm check`
- `pnpm test`
- `pnpm build`

Rust changes are accepted only after these local commands pass:

- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- `cargo test --manifest-path src-tauri/Cargo.toml`

When Rust contract types change, regenerate `src/shared/api/generated` with `ts-rs` and verify that only the expected generated bindings changed. These commands and the generated-binding check are part of the task acceptance criteria; do not report the task complete while any of them fail.

### Issue tracker

Issues are tracked as local Markdown files under `.scratch/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Triage uses the five default canonical label strings. See `docs/agents/triage-labels.md`.

### Domain docs

Domain documentation uses the single-context layout. See `docs/agents/domain.md`.
