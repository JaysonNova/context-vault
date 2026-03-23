# Context Vault MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a local macOS Tauri desktop app that scans CodeX, Claude Code, and Cursor conversations on startup or manual sync, stores them in SQLite, supports local search, and generates on-demand technical notes through a cloud API.

**Architecture:** Tauri hosts a React frontend and a Rust backend. Rust owns filesystem scanning, source adapters, SQLite persistence, FTS search, sync logs, Keychain integration, and note generation. React owns archive browsing, detail viewing, sync control, settings, and note reading. The data model stays workspace-aware without making workspace the primary V1 navigation model.

**Tech Stack:** Tauri 2, React 18, TypeScript, Vite, Vitest, Rust, rusqlite, serde, keyring, SQLite FTS5, npm

**Execution Rules:** Use `@test-driven-development` on every production change. Use `@verification-before-completion` before each commit and before claiming any task is finished.

---

## Planned File Structure

### Root Tooling

- Create: `package.json`
- Create: `tsconfig.json`
- Create: `tsconfig.node.json`
- Create: `vite.config.ts`
- Create: `vitest.setup.ts`
- Create: `index.html`
- Create: `README.md`

### Frontend App

- Create: `src/main.tsx`
- Create: `src/App.tsx`
- Create: `src/styles.css`
- Create: `src/lib/types.ts`
- Create: `src/lib/api.ts`
- Create: `src/lib/format.ts`
- Create: `src/components/layout/AppShell.tsx`
- Create: `src/components/layout/Sidebar.tsx`
- Create: `src/components/layout/TopBar.tsx`
- Create: `src/features/archive/pages/ArchivePage.tsx`
- Create: `src/features/archive/components/ConversationList.tsx`
- Create: `src/features/archive/components/ConversationFilters.tsx`
- Create: `src/features/archive/components/ConversationDetailPane.tsx`
- Create: `src/features/notes/pages/NotesPage.tsx`
- Create: `src/features/sync/pages/SyncLogPage.tsx`
- Create: `src/features/settings/pages/SettingsPage.tsx`

### Rust Backend

- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `src-tauri/src/error.rs`
- Create: `src-tauri/src/models.rs`
- Create: `src-tauri/src/export.rs`
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/archive.rs`
- Create: `src-tauri/src/commands/sync.rs`
- Create: `src-tauri/src/commands/notes.rs`
- Create: `src-tauri/src/commands/settings.rs`
- Create: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/db/connection.rs`
- Create: `src-tauri/src/db/migrations.rs`
- Create: `src-tauri/src/db/search.rs`
- Create: `src-tauri/src/sync/mod.rs`
- Create: `src-tauri/src/sync/normalize.rs`
- Create: `src-tauri/src/sync/runner.rs`
- Create: `src-tauri/src/adapters/mod.rs`
- Create: `src-tauri/src/adapters/claude_code.rs`
- Create: `src-tauri/src/adapters/codex.rs`
- Create: `src-tauri/src/adapters/cursor.rs`
- Create: `src-tauri/src/notes/mod.rs`
- Create: `src-tauri/src/notes/provider.rs`
- Create: `src-tauri/src/notes/prompt.rs`
- Create: `src-tauri/src/settings/mod.rs`
- Create: `src-tauri/src/settings/keychain.rs`
- Create: `src-tauri/src/settings/store.rs`

### SQL and Prompt Assets

- Create: `src-tauri/migrations/0001_init.sql`
- Create: `src-tauri/migrations/0002_fts.sql`
- Create: `src-tauri/prompts/technical_note_v1.md`

### Fixtures and Tests

- Create: `fixtures/sources/claude/projects/sample-session.jsonl`
- Create: `fixtures/sources/codex/history.jsonl`
- Create: `fixtures/sources/codex/thread_rows.json`
- Create: `fixtures/sources/cursor/composer_data.json`
- Create: `fixtures/sources/cursor/generations.json`
- Create: `src/components/layout/AppShell.test.tsx`
- Create: `src/features/archive/pages/ArchivePage.test.tsx`
- Create: `src/features/sync/pages/SyncLogPage.test.tsx`
- Create: `src/features/notes/pages/NotesPage.test.tsx`
- Create: `src-tauri/tests/db_schema.rs`
- Create: `src-tauri/tests/archive_commands.rs`
- Create: `src-tauri/tests/sync_runner.rs`
- Create: `src-tauri/tests/claude_adapter.rs`
- Create: `src-tauri/tests/codex_adapter.rs`
- Create: `src-tauri/tests/cursor_adapter.rs`
- Create: `src-tauri/tests/note_generation.rs`

## Task 1: Bootstrap Frontend Shell and Test Harness

**Files:**
- Create: `package.json`
- Create: `tsconfig.json`
- Create: `tsconfig.node.json`
- Create: `vite.config.ts`
- Create: `vitest.setup.ts`
- Create: `index.html`
- Create: `src/main.tsx`
- Create: `src/App.tsx`
- Create: `src/styles.css`
- Create: `src/components/layout/AppShell.tsx`
- Create: `src/components/layout/Sidebar.tsx`
- Create: `src/components/layout/TopBar.tsx`
- Test: `src/components/layout/AppShell.test.tsx`

- [ ] **Step 1: Create the frontend test harness and install dependencies**

Create `package.json` with scripts and dependencies only:

```json
{
  "name": "context-vault",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc -b && vite build",
    "test": "vitest run",
    "test:watch": "vitest",
    "tauri": "tauri"
  }
}
```

Install exact dependencies:

```bash
npm install react react-dom @tauri-apps/api
npm install -D typescript vite vitest jsdom @vitejs/plugin-react @testing-library/react @testing-library/jest-dom @types/react @types/react-dom @tauri-apps/cli
```

Expected: dependencies install successfully and `package-lock.json` is created

- [ ] **Step 2: Write the failing app shell test**

```tsx
import { render, screen } from '@testing-library/react';
import AppShell from './AppShell';

it('renders the primary navigation and sync action', () => {
  render(<AppShell />);

  expect(screen.getByText('对话记录')).toBeInTheDocument();
  expect(screen.getByText('知识笔记')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: '同步' })).toBeInTheDocument();
});
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `npm run test -- src/components/layout/AppShell.test.tsx`
Expected: FAIL with a module-not-found or render failure because `AppShell` does not exist yet

- [ ] **Step 4: Write the minimal frontend shell implementation**

Create a minimal shell that satisfies the smoke test:

```tsx
export default function AppShell() {
  return (
    <div className="app-shell">
      <Sidebar />
      <main />
      <TopBar />
    </div>
  );
}
```

Create `Sidebar.tsx` and `TopBar.tsx` as minimal presentational components, then wire the shell from `src/App.tsx` and `src/main.tsx`.

- [ ] **Step 5: Run the test and frontend build**

Run: `npm run test -- src/components/layout/AppShell.test.tsx`
Expected: PASS

Run: `npm run build`
Expected: Vite production build succeeds

- [ ] **Step 6: Commit**

```bash
git add package.json package-lock.json tsconfig.json tsconfig.node.json vite.config.ts vitest.setup.ts index.html src/main.tsx src/App.tsx src/styles.css src/components/layout/AppShell.tsx src/components/layout/Sidebar.tsx src/components/layout/TopBar.tsx src/components/layout/AppShell.test.tsx
git commit -m "chore: bootstrap frontend shell"
```

## Task 2: Scaffold Tauri Backend and SQLite Schema

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `src-tauri/src/error.rs`
- Create: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/db/connection.rs`
- Create: `src-tauri/src/db/migrations.rs`
- Create: `src-tauri/migrations/0001_init.sql`
- Create: `src-tauri/migrations/0002_fts.sql`
- Test: `src-tauri/tests/db_schema.rs`

- [ ] **Step 1: Write the failing database schema test**

```rust
#[test]
fn migrations_create_core_tables_and_fts() {
    let db = TestDb::new();
    db.run_migrations().unwrap();

    assert!(db.has_table("workspaces"));
    assert!(db.has_table("conversations"));
    assert!(db.has_table("messages"));
    assert!(db.has_table("notes"));
    assert!(db.has_table("sync_runs"));
    assert!(db.has_table("conversation_search"));
    assert!(db.has_table("note_search"));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml migrations_create_core_tables_and_fts -- --exact`
Expected: FAIL because the Tauri crate and migrations do not exist yet

- [ ] **Step 3: Implement the minimal Tauri crate and schema**

Create `src-tauri/Cargo.toml` with concrete dependencies:

```toml
[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
thiserror = "1"
anyhow = "1"
uuid = { version = "1", features = ["v4", "serde"] }
keyring = "3"
walkdir = "2"
glob = "0.3"
time = { version = "0.3", features = ["formatting", "macros"] }

[dev-dependencies]
tempfile = "3"
```

Define the initial SQL schema in `0001_init.sql`:

```sql
CREATE TABLE workspaces (
  id TEXT PRIMARY KEY,
  normalized_path TEXT NOT NULL UNIQUE,
  display_name TEXT NOT NULL,
  git_branch TEXT,
  git_origin_url TEXT,
  last_seen_at INTEGER NOT NULL
);

CREATE TABLE conversations (
  id TEXT PRIMARY KEY,
  source_app TEXT NOT NULL,
  source_conversation_id TEXT NOT NULL,
  workspace_id TEXT,
  title TEXT NOT NULL,
  subtitle TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  sync_strength TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'ready',
  raw_metadata_json TEXT NOT NULL,
  UNIQUE(source_app, source_conversation_id)
);
```

Add `messages`, `notes`, `note_sources`, `tags`, `entity_tags`, `sync_sources`, `sync_runs`, and `app_settings`.

Define FTS in `0002_fts.sql` with content tables and triggers:

```sql
CREATE VIRTUAL TABLE conversation_search USING fts5(
  conversation_id UNINDEXED,
  title,
  workspace_text,
  tag_text,
  message_text
);
```

- [ ] **Step 4: Run the Rust test suite**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS for schema tests

- [ ] **Step 5: Verify the desktop shell wiring compiles**

Run: `npm run tauri -- info`
Expected: Tauri CLI prints environment information without configuration errors

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/build.rs src-tauri/tauri.conf.json src-tauri/src/main.rs src-tauri/src/lib.rs src-tauri/src/error.rs src-tauri/src/db/mod.rs src-tauri/src/db/connection.rs src-tauri/src/db/migrations.rs src-tauri/migrations/0001_init.sql src-tauri/migrations/0002_fts.sql src-tauri/tests/db_schema.rs
git commit -m "feat: add tauri sqlite foundation"
```

## Task 3: Add Shared Models and Archive Query Commands

**Files:**
- Create: `src-tauri/src/models.rs`
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/archive.rs`
- Create: `src-tauri/src/commands/sync.rs`
- Create: `src/lib/types.ts`
- Create: `src/lib/api.ts`
- Create: `src/lib/format.ts`
- Create: `src/features/archive/pages/ArchivePage.tsx`
- Create: `src/features/archive/components/ConversationList.tsx`
- Create: `src/features/archive/components/ConversationDetailPane.tsx`
- Test: `src-tauri/tests/archive_commands.rs`
- Test: `src/features/archive/pages/ArchivePage.test.tsx`

- [ ] **Step 1: Write the failing backend archive query test**

```rust
#[test]
fn list_conversations_returns_rows_sorted_by_updated_at() {
    let db = seed_archive_db();
    let rows = archive::list_conversations(&db, ArchiveQuery::default()).unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].title, "gRPC 连接池耗尽排查");
}
```

- [ ] **Step 2: Write the failing frontend empty-state test**

```tsx
it('renders an empty-state prompt when no conversations exist', async () => {
  mockListConversations([]);
  render(<ArchivePage />);

  expect(await screen.findByText('还没有同步任何对话')).toBeInTheDocument();
});
```

- [ ] **Step 3: Run both tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml list_conversations_returns_rows_sorted_by_updated_at -- --exact`
Expected: FAIL because archive query commands are missing

Run: `npm run test -- src/features/archive/pages/ArchivePage.test.tsx`
Expected: FAIL because `ArchivePage` and Tauri API wrappers are missing

- [ ] **Step 4: Implement minimal archive models and commands**

Rust response shape:

```rust
#[derive(Serialize)]
pub struct ConversationListItem {
    pub id: String,
    pub title: String,
    pub source_app: String,
    pub sync_strength: String,
    pub workspace_name: Option<String>,
    pub updated_at: i64,
    pub note_count: i64,
}
```

TypeScript mirror:

```ts
export type ConversationListItem = {
  id: string;
  title: string;
  sourceApp: 'codex' | 'claude_code' | 'cursor';
  syncStrength: 'full' | 'partial' | 'metadata_only';
  workspaceName?: string;
  updatedAt: number;
  noteCount: number;
};
```

- [ ] **Step 5: Run the targeted tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml archive_commands`
Expected: PASS

Run: `npm run test -- src/features/archive/pages/ArchivePage.test.tsx`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/models.rs src-tauri/src/commands/mod.rs src-tauri/src/commands/archive.rs src-tauri/src/commands/sync.rs src-tauri/tests/archive_commands.rs src/lib/types.ts src/lib/api.ts src/lib/format.ts src/features/archive/pages/ArchivePage.tsx src/features/archive/components/ConversationList.tsx src/features/archive/components/ConversationDetailPane.tsx src/features/archive/pages/ArchivePage.test.tsx
git commit -m "feat: add archive query layer"
```

## Task 4: Implement the Claude Code Adapter

**Files:**
- Create: `fixtures/sources/claude/projects/sample-session.jsonl`
- Create: `src-tauri/src/adapters/mod.rs`
- Create: `src-tauri/src/adapters/claude_code.rs`
- Create: `src-tauri/src/sync/normalize.rs`
- Test: `src-tauri/tests/claude_adapter.rs`

- [ ] **Step 1: Create a redacted Claude fixture and failing parser test**

Use a redacted JSONL sample with:

- one `user` message
- one `assistant` message
- one `tool_use`
- one `tool_result`

```rust
#[test]
fn claude_adapter_parses_full_conversation_with_tool_events() {
    let result = ClaudeCodeAdapter::parse_fixture("fixtures/sources/claude/projects/sample-session.jsonl").unwrap();

    assert_eq!(result.conversation.source_app, "claude_code");
    assert_eq!(result.conversation.sync_strength, "full");
    assert!(result.messages.iter().any(|m| m.message_type == "tool_call"));
    assert!(result.messages.iter().any(|m| m.message_type == "tool_result"));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml claude_adapter_parses_full_conversation_with_tool_events -- --exact`
Expected: FAIL because the adapter does not exist

- [ ] **Step 3: Implement the Claude parser**

Normalize source roles into internal message types:

```rust
match source_type.as_str() {
    "user" => ("user", "user"),
    "assistant" => ("assistant", "assistant"),
    "tool_use" => ("assistant", "tool_call"),
    "tool_result" => ("tool", "tool_result"),
    _ => ("system", "metadata"),
}
```

Set conversation metadata from session-level fields such as `cwd`, `sessionId`, `gitBranch`, and timestamps.

- [ ] **Step 4: Run the adapter and regression tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml claude_adapter`
Expected: PASS

Run: `cargo test --manifest-path src-tauri/Cargo.toml archive_commands`
Expected: PASS with no regression

- [ ] **Step 5: Commit**

```bash
git add fixtures/sources/claude/projects/sample-session.jsonl src-tauri/src/adapters/mod.rs src-tauri/src/adapters/claude_code.rs src-tauri/src/sync/normalize.rs src-tauri/tests/claude_adapter.rs
git commit -m "feat: import claude code conversations"
```

## Task 5: Implement the CodeX Adapter

**Files:**
- Create: `fixtures/sources/codex/history.jsonl`
- Create: `fixtures/sources/codex/thread_rows.json`
- Create: `src-tauri/src/adapters/codex.rs`
- Test: `src-tauri/tests/codex_adapter.rs`

- [ ] **Step 1: Create redacted CodeX fixtures and a failing adapter test**

Fixture contents:

- `history.jsonl` with multiple `session_id`, `ts`, `text` rows
- `thread_rows.json` with `id`, `title`, `cwd`, `git_branch`, `git_origin_url`

```rust
#[test]
fn codex_adapter_builds_partial_conversation_from_history_and_threads() {
    let parsed = CodexAdapter::parse_fixture_dir("fixtures/sources/codex").unwrap();

    assert_eq!(parsed.conversation.source_app, "codex");
    assert_eq!(parsed.conversation.sync_strength, "partial");
    assert!(parsed.messages.iter().all(|m| m.role == "user"));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml codex_adapter_builds_partial_conversation_from_history_and_threads -- --exact`
Expected: FAIL because the adapter does not exist

- [ ] **Step 3: Implement the minimal CodeX parser**

Combine history and thread metadata by session id:

```rust
let conversation = ConversationImport {
    source_app: "codex".into(),
    source_conversation_id: session_id.clone(),
    title: thread.title.unwrap_or_else(|| first_prompt_title(&messages)),
    sync_strength: "partial".into(),
    raw_metadata_json: serde_json::to_string(&thread)?,
};
```

Only import stable user timeline records in V1. Do not attempt speculative reconstruction of assistant transcript.

- [ ] **Step 4: Run adapter tests and a sync smoke test**

Run: `cargo test --manifest-path src-tauri/Cargo.toml codex_adapter`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add fixtures/sources/codex/history.jsonl fixtures/sources/codex/thread_rows.json src-tauri/src/adapters/codex.rs src-tauri/tests/codex_adapter.rs
git commit -m "feat: import codex conversations"
```

## Task 6: Implement the Cursor Adapter

**Files:**
- Create: `fixtures/sources/cursor/composer_data.json`
- Create: `fixtures/sources/cursor/generations.json`
- Create: `src-tauri/src/adapters/cursor.rs`
- Test: `src-tauri/tests/cursor_adapter.rs`

- [ ] **Step 1: Create the failing Cursor adapter test**

```rust
#[test]
fn cursor_adapter_imports_metadata_only_conversations() {
    let parsed = CursorAdapter::parse_fixture_dir("fixtures/sources/cursor").unwrap();

    assert_eq!(parsed.conversation.source_app, "cursor");
    assert_eq!(parsed.conversation.sync_strength, "metadata_only");
    assert!(parsed.messages.iter().is_empty() || parsed.messages.iter().all(|m| m.message_type == "metadata"));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml cursor_adapter_imports_metadata_only_conversations -- --exact`
Expected: FAIL because the adapter does not exist

- [ ] **Step 3: Implement Cursor metadata extraction**

Map stable composer metadata only:

```rust
let conversation = ConversationImport {
    source_app: "cursor".into(),
    source_conversation_id: composer_id.clone(),
    title: composer.name.unwrap_or_else(|| "Untitled Cursor Composer".into()),
    subtitle: composer.subtitle,
    sync_strength: "metadata_only".into(),
    raw_metadata_json: serde_json::to_string(&composer)?,
};
```

Attach prompt fragments from `aiService.generations` as optional metadata rows only when the `composerId` can be linked safely.

- [ ] **Step 4: Run the adapter tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml cursor_adapter`
Expected: PASS

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: all current Rust tests PASS

- [ ] **Step 5: Commit**

```bash
git add fixtures/sources/cursor/composer_data.json fixtures/sources/cursor/generations.json src-tauri/src/adapters/cursor.rs src-tauri/tests/cursor_adapter.rs
git commit -m "feat: import cursor metadata"
```

## Task 7: Build the Sync Runner, Startup Sync, and Sync Log View

**Files:**
- Create: `src-tauri/src/sync/mod.rs`
- Create: `src-tauri/src/sync/runner.rs`
- Modify: `src-tauri/src/commands/sync.rs`
- Create: `src/features/sync/pages/SyncLogPage.tsx`
- Create: `src/features/sync/pages/SyncLogPage.test.tsx`
- Modify: `src/App.tsx`
- Modify: `src/lib/api.ts`

- [ ] **Step 1: Write the failing sync runner test**

```rust
#[test]
fn sync_runner_records_partial_success_per_adapter() {
    let result = run_sync_with_stub_adapters(vec![
        Ok(SyncImportCount { conversations: 2, messages: 8 }),
        Err(anyhow!("cursor db locked")),
    ]).unwrap();

    assert_eq!(result.status, "partial_success");
    assert_eq!(result.source_results.len(), 2);
}
```

- [ ] **Step 2: Write the failing sync log UI test**

```tsx
it('shows per-source sync results', async () => {
  mockListSyncRuns([
    { sourceApp: 'claude_code', status: 'success', importedConversationCount: 3 },
    { sourceApp: 'cursor', status: 'failed', errorSummary: 'db locked' }
  ]);

  render(<SyncLogPage />);

  expect(await screen.findByText('claude_code')).toBeInTheDocument();
  expect(screen.getByText('db locked')).toBeInTheDocument();
});
```

- [ ] **Step 3: Run both tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml sync_runner_records_partial_success_per_adapter -- --exact`
Expected: FAIL because the runner does not exist

Run: `npm run test -- src/features/sync/pages/SyncLogPage.test.tsx`
Expected: FAIL because the page does not exist

- [ ] **Step 4: Implement the sync runner and UI wiring**

Rust sync result shape:

```rust
pub struct SyncRunResult {
    pub id: String,
    pub status: String,
    pub source_results: Vec<SourceSyncResult>,
}
```

Frontend startup behavior:

```tsx
useEffect(() => {
  void runSync({ trigger: 'startup' });
}, []);
```

The runner must:

- call adapters independently
- persist `sync_runs`
- persist per-source warnings and failures
- return a user-readable result summary

- [ ] **Step 5: Run the tests and manual sync command**

Run: `cargo test --manifest-path src-tauri/Cargo.toml sync_runner`
Expected: PASS

Run: `npm run test -- src/features/sync/pages/SyncLogPage.test.tsx`
Expected: PASS

Run: `npm run tauri -- dev`
Expected: app launches and triggers startup sync without crashing

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/sync/mod.rs src-tauri/src/sync/runner.rs src-tauri/src/commands/sync.rs src/features/sync/pages/SyncLogPage.tsx src/features/sync/pages/SyncLogPage.test.tsx src/App.tsx src/lib/api.ts
git commit -m "feat: add sync runner and logs"
```

## Task 8: Implement Archive Search, Filters, and Detail View

**Files:**
- Create: `src-tauri/src/db/search.rs`
- Modify: `src-tauri/src/commands/archive.rs`
- Create: `src/features/archive/components/ConversationFilters.tsx`
- Modify: `src/features/archive/components/ConversationList.tsx`
- Modify: `src/features/archive/components/ConversationDetailPane.tsx`
- Modify: `src/features/archive/pages/ArchivePage.tsx`
- Test: `src/features/archive/pages/ArchivePage.test.tsx`

- [ ] **Step 1: Extend the failing archive UI test**

```tsx
it('filters by sync strength and renders the selected conversation detail', async () => {
  mockListConversations([
    { id: '1', title: 'Claude Full', syncStrength: 'full' },
    { id: '2', title: 'CodeX Partial', syncStrength: 'partial' }
  ]);

  render(<ArchivePage />);
  await user.click(await screen.findByRole('button', { name: 'partial' }));
  await user.click(screen.getByText('CodeX Partial'));

  expect(screen.getByText('partial')).toBeInTheDocument();
  expect(screen.getByText('CodeX Partial')).toBeInTheDocument();
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `npm run test -- src/features/archive/pages/ArchivePage.test.tsx`
Expected: FAIL because filter state and detail selection are incomplete

- [ ] **Step 3: Implement local search and filtering**

Rust query shape:

```rust
pub struct ArchiveQuery {
    pub text: Option<String>,
    pub source_app: Option<String>,
    pub sync_strength: Option<String>,
    pub workspace_id: Option<String>,
    pub note_state: Option<String>,
}
```

Detail pane requirements:

- always show `full`, `partial`, or `metadata_only`
- show imported timeline when available
- show metadata summary when timeline is limited

- [ ] **Step 4: Run frontend tests and a targeted Rust search test**

Run: `npm run test -- src/features/archive/pages/ArchivePage.test.tsx`
Expected: PASS

Run: `cargo test --manifest-path src-tauri/Cargo.toml archive_search`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/db/search.rs src-tauri/src/commands/archive.rs src/features/archive/components/ConversationFilters.tsx src/features/archive/components/ConversationList.tsx src/features/archive/components/ConversationDetailPane.tsx src/features/archive/pages/ArchivePage.tsx src/features/archive/pages/ArchivePage.test.tsx
git commit -m "feat: add archive search and detail"
```

## Task 9: Add Settings, Keychain Storage, and On-Demand Note Generation

**Files:**
- Create: `src-tauri/src/settings/mod.rs`
- Create: `src-tauri/src/settings/keychain.rs`
- Create: `src-tauri/src/settings/store.rs`
- Create: `src-tauri/src/notes/mod.rs`
- Create: `src-tauri/src/notes/provider.rs`
- Create: `src-tauri/src/notes/prompt.rs`
- Create: `src-tauri/prompts/technical_note_v1.md`
- Create: `src-tauri/src/commands/settings.rs`
- Create: `src-tauri/src/commands/notes.rs`
- Create: `src/features/settings/pages/SettingsPage.tsx`
- Create: `src/features/notes/pages/NotesPage.tsx`
- Modify: `src/App.tsx`
- Modify: `src/lib/api.ts`
- Test: `src-tauri/tests/note_generation.rs`
- Test: `src/features/notes/pages/NotesPage.test.tsx`

- [ ] **Step 1: Write the failing note generation backend test**

```rust
#[test]
fn generate_note_persists_summary_body_and_prompt_version() {
    let db = seed_conversation_for_note();
    let provider = StubProvider::success("# Note\n\nSummary");

    let note = generate_note(&db, provider, "conv_1").unwrap();

    assert_eq!(note.prompt_version, "technical_note_v1");
    assert!(note.body_md.contains("# Note"));
}
```

- [ ] **Step 2: Write the failing notes page test**

```tsx
it('renders generated notes and export action', async () => {
  mockListNotes([{ id: 'n1', title: 'gRPC 排障笔记', summary: '连接池上限问题' }]);
  render(<NotesPage />);

  expect(await screen.findByText('gRPC 排障笔记')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: '导出 Markdown' })).toBeInTheDocument();
});
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml generate_note_persists_summary_body_and_prompt_version -- --exact`
Expected: FAIL because note generation is not implemented

Run: `npm run test -- src/features/notes/pages/NotesPage.test.tsx`
Expected: FAIL because the notes page is not implemented

- [ ] **Step 4: Implement settings, provider, and note generation**

Store non-secret settings locally:

```rust
pub struct AppSettings {
    pub provider: String,
    pub model: String,
    pub base_url: Option<String>,
}
```

Store the secret separately:

```rust
keyring::Entry::new("context-vault", "llm-api-key")?.set_password(api_key)?;
```

Prompt version contract:

```rust
pub const PROMPT_VERSION: &str = "technical_note_v1";
```

The note generation command must:

- load conversation plus messages
- reject generation when no API key exists
- call the configured provider only on explicit user action
- write `notes` and `note_sources`

- [ ] **Step 5: Run backend and frontend tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml note_generation`
Expected: PASS

Run: `npm run test -- src/features/notes/pages/NotesPage.test.tsx`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/settings/mod.rs src-tauri/src/settings/keychain.rs src-tauri/src/settings/store.rs src-tauri/src/notes/mod.rs src-tauri/src/notes/provider.rs src-tauri/src/notes/prompt.rs src-tauri/prompts/technical_note_v1.md src-tauri/src/commands/settings.rs src-tauri/src/commands/notes.rs src-tauri/tests/note_generation.rs src/features/settings/pages/SettingsPage.tsx src/features/notes/pages/NotesPage.tsx src/features/notes/pages/NotesPage.test.tsx src/App.tsx src/lib/api.ts
git commit -m "feat: add note generation and settings"
```

## Task 10: Add Markdown Export, README, and Full Verification

**Files:**
- Create: `src-tauri/src/export.rs`
- Modify: `src-tauri/src/commands/notes.rs`
- Modify: `src/features/notes/pages/NotesPage.tsx`
- Create: `README.md`
- Modify: `src/App.tsx`

- [ ] **Step 1: Write the failing export test**

```rust
#[test]
fn export_note_writes_markdown_to_requested_path() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("grpc-note.md");

    export_note_markdown(&db, "note_1", &path).unwrap();

    let body = std::fs::read_to_string(path).unwrap();
    assert!(body.contains("# gRPC 排障笔记"));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml export_note_writes_markdown_to_requested_path -- --exact`
Expected: FAIL because export is not implemented

- [ ] **Step 3: Implement markdown export and polish the README**

Export function:

```rust
pub fn export_note_markdown(db: &Connection, note_id: &str, path: &Path) -> Result<()> {
    let note = load_note(db, note_id)?;
    std::fs::write(path, note.body_md)?;
    Ok(())
}
```

`README.md` must include:

- local prerequisites
- install command
- dev command
- test commands
- source directories scanned by V1
- honesty note about `full`, `partial`, and `metadata_only`

- [ ] **Step 4: Run the complete verification suite**

Run: `npm run test`
Expected: all frontend tests PASS

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: all Rust tests PASS

Run: `npm run build`
Expected: frontend production build PASS

Run: `npm run tauri -- build`
Expected: macOS desktop build PASS

- [ ] **Step 5: Manual smoke test**

Launch: `npm run tauri -- dev`

Manual checklist:

- startup sync imports at least one source
- sync log records the run
- archive search filters results
- detail pane shows completeness label
- note generation works with a valid API key
- generated note appears in notes view
- markdown export writes a file successfully

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/export.rs src-tauri/src/commands/notes.rs src/features/notes/pages/NotesPage.tsx README.md src/App.tsx
git commit -m "feat: complete context vault mvp"
```

## Implementation Notes

- Keep source adapter parsing conservative. If a source record is ambiguous, preserve it as metadata instead of inventing a transcript.
- Keep all timestamps in Unix milliseconds in the normalized model.
- Use fixture-based tests before pointing at live source directories.
- Do not store secrets in SQLite, JSON files, or environment files committed to the repo.
- Prefer simple React state and explicit prop flow over introducing extra client state libraries in V1.
- Do not add background watchers, menu bar mode, or embeddings in this plan.
