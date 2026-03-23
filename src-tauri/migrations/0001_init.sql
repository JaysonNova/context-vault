PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS workspaces (
  id TEXT PRIMARY KEY,
  normalized_path TEXT NOT NULL UNIQUE,
  display_name TEXT NOT NULL,
  git_branch TEXT,
  git_origin_url TEXT,
  last_seen_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS conversations (
  id TEXT PRIMARY KEY,
  source_app TEXT NOT NULL,
  source_conversation_id TEXT NOT NULL,
  workspace_id TEXT REFERENCES workspaces(id) ON DELETE SET NULL,
  title TEXT NOT NULL,
  subtitle TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  sync_strength TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'ready',
  raw_metadata_json TEXT NOT NULL,
  UNIQUE(source_app, source_conversation_id)
);

CREATE TABLE IF NOT EXISTS messages (
  id TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
  source_message_id TEXT,
  role TEXT NOT NULL,
  message_type TEXT NOT NULL,
  content_text TEXT NOT NULL,
  tool_name TEXT,
  token_count INTEGER,
  created_at INTEGER NOT NULL,
  raw_payload_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS notes (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  summary TEXT NOT NULL,
  body_md TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'ready',
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  model_provider TEXT,
  model_name TEXT,
  prompt_version TEXT
);

CREATE TABLE IF NOT EXISTS note_sources (
  note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
  conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
  PRIMARY KEY (note_id, conversation_id)
);

CREATE TABLE IF NOT EXISTS tags (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  source_type TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS entity_tags (
  entity_type TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
  PRIMARY KEY (entity_type, entity_id, tag_id)
);

CREATE TABLE IF NOT EXISTS sync_sources (
  id TEXT PRIMARY KEY,
  source_app TEXT NOT NULL UNIQUE,
  config_json TEXT NOT NULL DEFAULT '{}',
  watermark_json TEXT NOT NULL DEFAULT '{}',
  updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS sync_runs (
  id TEXT PRIMARY KEY,
  started_at INTEGER NOT NULL,
  finished_at INTEGER,
  source_app TEXT NOT NULL,
  status TEXT NOT NULL,
  imported_conversation_count INTEGER NOT NULL DEFAULT 0,
  imported_message_count INTEGER NOT NULL DEFAULT 0,
  warning_count INTEGER NOT NULL DEFAULT 0,
  error_summary TEXT
);

CREATE TABLE IF NOT EXISTS app_settings (
  key TEXT PRIMARY KEY,
  value_json TEXT NOT NULL
);
