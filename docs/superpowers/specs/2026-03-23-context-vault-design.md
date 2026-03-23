# Context Vault Design

## Overview

Context Vault is a local macOS desktop app for collecting AI coding conversations from multiple tools, organizing them in one place, and turning selected conversations into searchable technical notes.

The first release is optimized for:

- local use on macOS
- startup scan plus manual sync
- on-demand note generation
- strong support for CodeX and Claude Code first, with weaker Cursor support

The product goal is not to perfectly replay every upstream chat product. The goal is to create a reliable local archive and knowledge workflow that is honest about what data was captured and how complete it is.

## Product Scope

### In Scope

- local desktop app with a native-feeling macOS window
- startup scan when the app launches
- manual sync button for re-running import
- unified archive for CodeX, Claude Code, and Cursor conversations
- searchable conversations and generated notes
- on-demand note generation through a cloud LLM API
- local persistence of raw imported data and generated notes
- source completeness labels such as `full`, `partial`, and `metadata_only`

### Out of Scope for V1

- background resident sync
- real-time filesystem watching
- automatic note generation
- team sharing
- cloud sync across devices
- formal app sandboxing or App Store distribution
- guaranteed full-fidelity replay for every source

## User Experience

### Primary Flow

1. User opens the app.
2. The app scans configured local source directories once.
3. User browses imported conversations in the archive view.
4. User filters by source, project, completeness, or note status.
5. User opens a conversation detail page.
6. User clicks `Generate Note` on a conversation worth preserving.
7. The app sends that conversation to the configured cloud model.
8. The generated note is stored locally and becomes searchable in the notes view.

### Core Screens

#### 1. Conversation Archive

Default landing screen.

Purpose:

- browse imported conversations quickly
- identify valuable conversations
- expose sync completeness honestly

Key UI elements:

- source counts in the sidebar
- search box
- filters for source, workspace, date range, completeness, and note status
- high-density conversation list with title, source, workspace, updated time, tags, and completeness badge
- sync action in the top bar

#### 2. Conversation Detail

Purpose:

- inspect the imported conversation
- understand how complete the import is
- trigger note generation

Key UI elements:

- conversation header with title, source, completeness, workspace, timestamps
- raw or normalized timeline, depending on source fidelity
- imported metadata block
- `Generate Note` action
- generated note preview when available

#### 3. Notes View

Purpose:

- browse and search curated knowledge
- separate generated notes from raw conversations

Key UI elements:

- note list with title, summary, tags, linked source conversations, updated time
- full text search
- markdown detail or reader view
- export as Markdown

#### 4. Sync Log View

Purpose:

- make sync results observable
- isolate source-specific failures from the main browsing experience

Key UI elements:

- sync run history
- imported counts by source
- error summaries by adapter
- warnings for unreadable files or unsupported source records

## Technical Architecture

The app should use a layered local-first architecture:

1. `Desktop Shell`
   Tauri window, system integration, keychain access, and local storage path handling.
2. `Frontend App`
   React UI for archive, detail, notes, sync logs, and settings.
3. `Ingestion Core`
   Orchestrates adapters, normalization, watermarking, deduplication, and sync runs.
4. `Local Data Store`
   SQLite database with FTS support.
5. `AI Note Generation Service`
   Invoked only when the user explicitly requests note generation.

### Recommended Stack

- desktop shell: Tauri
- frontend: React + TypeScript
- styling: lightweight utility-first or component styling, chosen for speed and consistency
- persistence: SQLite
- search: SQLite FTS5
- backend logic inside app: Rust or TypeScript via Tauri commands, depending on implementation ergonomics
- secure secret storage: macOS Keychain through Tauri plugin or native integration

## Source Adapters

The adapters are intentionally separate from UI logic. Each adapter reads source-specific files, produces normalized records, and records sync metadata.

### 1. CodeX Adapter

Primary source paths:

- `~/.codex/history.jsonl`
- `~/.codex/state_*.sqlite`

Expected V1 fidelity:

- stable user prompt timeline from `history.jsonl`
- stable thread metadata from sqlite
- workspace and git context when available

V1 completeness classification:

- usually `partial`

Reason:

- user messages and thread metadata are available and reliable
- assistant and tool transcript reconstruction may require deeper source reverse engineering and should not be promised in V1

### 2. Claude Code Adapter

Primary source paths:

- `~/.claude/projects/**/*.jsonl`

Expected V1 fidelity:

- near-complete conversation replay
- user, assistant, tool use, tool result, cwd, branch, model, and timestamps

V1 completeness classification:

- usually `full`

Reason:

- source data is already stored as rich structured JSONL

### 3. Cursor Adapter

Primary source paths:

- `~/Library/Application Support/Cursor/User/workspaceStorage/**/state.vscdb`

Expected V1 fidelity:

- composer metadata
- titles
- timestamps
- branch or workspace hints when present
- prompt fragments and change summary fields when present

V1 completeness classification:

- `metadata_only` or selective `partial`

Reason:

- stable access exists for composer-level metadata
- full transcript recovery is not yet reliable enough for V1

## Sync Model

### Sync Triggering

V1 sync modes:

- automatic scan once at app startup
- manual sync by explicit user action

No background process should remain active after the main window is closed.

### Sync Strategy

- each adapter runs independently
- adapter failure must not block the other adapters
- each adapter maintains its own watermark strategy
- sync runs are persisted for observability

### Watermarks and Incremental Rules

#### CodeX

- track file offset for `history.jsonl`
- upsert thread metadata by source thread id and updated timestamp

#### Claude Code

- track known file paths and processed message ids
- rely on append-friendly JSONL behavior

#### Cursor

- rescan known workspace databases
- upsert by `composerId`
- update only when `lastUpdatedAt` changes

### Deduplication

Deduplication must be source-specific and deterministic.

Recommended identifiers:

- CodeX: `source_app + session_id + ts + content_hash`
- Claude Code: `source_app + sessionId + uuid`
- Cursor: `source_app + composerId`

## Data Model

The schema should support V1 while leaving room for later workspace-first organization.

### `workspaces`

Stores project-level grouping.

Suggested fields:

- `id`
- `normalized_path`
- `display_name`
- `git_branch`
- `git_origin_url`
- `last_seen_at`

### `conversations`

Stores normalized conversation headers.

Suggested fields:

- `id`
- `source_app`
- `source_conversation_id`
- `workspace_id`
- `title`
- `subtitle`
- `created_at`
- `updated_at`
- `sync_strength`
- `status`
- `raw_metadata_json`

### `messages`

Stores normalized timeline entries.

Suggested fields:

- `id`
- `conversation_id`
- `source_message_id`
- `role`
- `message_type`
- `content_text`
- `tool_name`
- `token_count`
- `created_at`
- `raw_payload_json`

`message_type` must support more than simple chat roles. It should allow values such as:

- `user`
- `assistant`
- `tool_call`
- `tool_result`
- `system`
- `thinking`
- `metadata`

### `notes`

Stores generated technical notes.

Suggested fields:

- `id`
- `title`
- `summary`
- `body_md`
- `status`
- `created_at`
- `updated_at`
- `model_provider`
- `model_name`
- `prompt_version`

### `note_sources`

Maps notes to one or more conversations.

This avoids forcing a one-note-per-conversation relationship.

### `tags`

Stores tags with provenance.

Suggested tag source types:

- `manual`
- `ai_generated`
- `imported`

### `sync_sources`

Stores source configuration and watermarks.

### `sync_runs`

Stores each sync execution result.

Suggested fields:

- `id`
- `started_at`
- `finished_at`
- `source_app`
- `status`
- `imported_conversation_count`
- `imported_message_count`
- `warning_count`
- `error_summary`

## Search Design

Search is one of the main product values, so V1 should use real local full-text search instead of basic string filtering.

### Search Targets

Search across:

- conversation titles
- workspace names or paths
- tags
- imported message text where available
- generated note summaries
- generated note bodies

### Search Technology

Use SQLite FTS5 for V1.

Reasons:

- local-first
- fast enough for expected volume
- low operational complexity
- avoids introducing a separate search service

### Filtering

V1 filters should be intentionally small:

- source app
- sync completeness
- workspace or project
- date range
- note generated or not

## AI Note Generation

### Trigger Model

V1 is strictly on-demand.

The app must not automatically send newly imported conversations to the model.

### Provider Model

The app should support cloud API providers first.

Expected setup:

- user enters API credentials in settings
- credentials are stored securely in Keychain
- per-note generation uses the selected provider and model

### Input Policy

Only the selected conversation should be sent to the model.

The app should not silently include unrelated conversations or global history.

### Expected Output

Generated note structure should include:

- concise summary
- problem statement
- root cause or conceptual explanation
- solution or implementation steps
- key takeaways
- suggested tags
- markdown body suitable for later review

### Failure Handling

- generation failure must not corrupt conversation data
- failed generations should show retry affordance
- repeated clicks should not create duplicate notes unless the user explicitly regenerates

## Completeness and Trust Model

V1 must be honest about import fidelity.

Every conversation should display one of these states:

- `full`
- `partial`
- `metadata_only`

This is a product requirement, not a cosmetic label. The user must always know whether the detail view is showing a full imported conversation or a best-effort archive.

## Privacy and Security

### Local Data

- imported source data remains local by default
- generated notes remain local by default
- no automatic cloud sync

### Cloud Calls

- only user-triggered note generation sends content to a model provider
- the UI should make this explicit before first use

### Secrets

- API keys stored in Keychain
- do not store raw secrets in plaintext config files

### Source Controls

The settings screen should allow:

- disabling a source adapter
- excluding specific paths or workspaces
- clearing imported local data

## Error Handling

### Adapter Isolation

- one source failure must not fail the entire sync
- the UI should show partial success

### Parse Failures

- record the source path and parse error summary
- continue sync where safe

### Corrupt or Unsupported Records

- skip the record
- log the failure
- surface the count in sync logs

## Testing Strategy

### Unit Tests

- source-specific parsers
- normalization logic
- completeness classification
- deduplication keys

### Integration Tests

- startup sync
- re-sync idempotency
- SQLite write path
- note generation request and persistence flow
- FTS search behavior

### Fixture Strategy

Use local, redacted fixtures derived from real-world samples for:

- CodeX
- Claude Code
- Cursor

### UI Verification

Critical-path coverage for:

- launch and startup scan
- filtering archive results
- opening detail view
- generating a note
- viewing the note in the notes section

## Delivery Boundary for V1

V1 is complete when the following are true:

- the macOS desktop app launches locally
- startup scan imports CodeX, Claude Code, and Cursor metadata with source-specific fidelity
- imported conversations are stored locally and browsable
- local search works across conversations and notes
- a user can generate a note from a selected conversation using a cloud API
- generated notes persist locally and are searchable
- sync logs make adapter failures understandable
- the UI clearly distinguishes `full`, `partial`, and `metadata_only`

## Future Expansion

The schema and app structure should support later additions without a rewrite:

- workspace-first grouping as a primary navigation mode
- background sync or menu bar mode
- local model support
- richer Cursor import
- batch analysis
- embeddings or vector retrieval if local search later becomes insufficient

## Design Summary

The recommended V1 is a local-first Tauri desktop app with a strong ingestion core and a deliberately honest UI.

The most important design decisions are:

- archive first, summarize later
- startup scan plus manual sync only
- cloud note generation only on demand
- first-class support for uneven source fidelity
- workspace-aware schema without making workspace the primary V1 interaction model
