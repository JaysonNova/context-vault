# Repository Guidelines

## Project Structure & Module Organization

`src/` contains the React + Vite frontend. Feature pages live under `src/features/*/pages`, reusable UI sits in `src/components/`, and shared API/types helpers are in `src/lib/`. Frontend tests are colocated as `*.test.tsx`. `src-tauri/` contains the Rust desktop backend: Tauri commands under `src-tauri/src/commands`, adapters and sync logic under `src-tauri/src/adapters` and `src-tauri/src/sync`, migrations in `src-tauri/migrations`, and Rust integration tests in `src-tauri/tests/`. Keep deterministic sample inputs in `fixtures/sources/`.

## Build, Test, and Development Commands

- `npm install`: install Node dependencies.
- `npm run tauri -- dev`: run the desktop app locally with the Vite dev server.
- `npm run build`: type-check and build the frontend only.
- `npm run tauri -- build`: create a release desktop build.
- `npm run tauri -- build --bundles dmg`: build the macOS release DMG.
- `npm run test`: run frontend tests with Vitest.
- `cargo test --manifest-path src-tauri/Cargo.toml`: run Rust unit and integration tests.

Release bundles are written to `src-tauri/target/release/bundle/`.

## Coding Style & Naming Conventions

Match existing file-local style. In the current codebase, TypeScript/TSX uses 2-space indentation, PascalCase component names, and camelCase variables/functions. Rust follows standard `rustfmt` style with snake_case function/module names. Name tests `ComponentName.test.tsx` and keep them beside the component/page they cover. There is no dedicated lint script yet; treat `npm run build` and the test suites as the minimum quality gate.

## Testing Guidelines

Frontend tests use Vitest with Testing Library and `jsdom`; prefer user-visible assertions over implementation details. Rust behavior is covered in `src-tauri/tests/*.rs`; add or extend integration tests when changing commands, adapters, sync, export, or DB behavior. Run both `npm run test` and `cargo test --manifest-path src-tauri/Cargo.toml` before opening a PR that touches both layers.

## Commit & Pull Request Guidelines

Recent history uses conventional prefixes such as `feat:`, `fix:`, and `chore:` with short imperative summaries. Keep commits focused and scoped to one change. PRs should explain the user-visible effect, list verification commands you ran, and include screenshots for UI changes. Link the relevant issue or task when one exists.

## Security & Configuration Tips

This app reads local data from CodeX, Claude Code, and Cursor paths on macOS. Do not commit personal archives, database dumps, API secrets, or local machine paths beyond sanitized fixtures.
