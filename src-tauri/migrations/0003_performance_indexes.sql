CREATE INDEX IF NOT EXISTS idx_conversations_updated_at
  ON conversations(updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_messages_conversation_created_at
  ON messages(conversation_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_messages_conversation_type_created_at
  ON messages(conversation_id, message_type, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_note_sources_conversation_id
  ON note_sources(conversation_id);

CREATE INDEX IF NOT EXISTS idx_notes_updated_at
  ON notes(updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_sync_runs_started_at
  ON sync_runs(started_at DESC);
