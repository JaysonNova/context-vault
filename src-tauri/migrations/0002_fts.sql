CREATE VIRTUAL TABLE IF NOT EXISTS conversation_search USING fts5(
  conversation_id UNINDEXED,
  title,
  workspace_text,
  tag_text,
  message_text
);

CREATE VIRTUAL TABLE IF NOT EXISTS note_search USING fts5(
  note_id UNINDEXED,
  title,
  summary,
  body_md,
  tag_text
);
