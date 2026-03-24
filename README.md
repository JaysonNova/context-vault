# Context Vault

Context Vault is a local macOS Tauri app for collecting AI coding conversations from CodeX, Claude Code, and Cursor, then turning selected conversations into searchable technical notes.

## Prerequisites

- Node.js 22+
- npm 11+
- Rust stable toolchain
- Xcode Command Line Tools on macOS

## Development

Common commands:

```bash
# Install dependencies
npm install

# Run all web tests
npm run test

# Run all Rust tests
cargo test --manifest-path src-tauri/Cargo.toml

# Start the desktop app in development
npm run tauri -- dev

# Build the frontend only
npm run build

# Build the desktop app in release mode
npm run tauri -- build

# Build only the macOS DMG installer in release mode
npm run tauri -- build --bundles dmg
```

Bundle outputs are generated under `src-tauri/target/release/bundle/`.

## V1 Source Coverage

V1 scans these local sources:

- `~/.codex/history.jsonl` and CodeX thread metadata
- `~/.claude/projects/**/*.jsonl`
- `~/Library/Application Support/Cursor/User/workspaceStorage/**/state.vscdb`

## Completeness Labels

Context Vault is explicit about import fidelity:

- `full`: conversation timeline is close to complete
- `partial`: only a reliable subset is imported
- `metadata_only`: title, workspace hints, and related metadata are available, but not a trustworthy transcript

These labels are intentional. V1 prefers honest archives over fake completeness.
