use rusqlite::Connection;

use context_vault_lib::db::migrations::run_migrations;

struct TestDb {
    conn: Connection,
}

impl TestDb {
    fn new() -> Self {
        Self {
            conn: Connection::open_in_memory().unwrap(),
        }
    }

    fn run_migrations(&self) {
        run_migrations(&self.conn).unwrap();
    }

    fn has_table(&self, table_name: &str) -> bool {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type IN ('table', 'view') AND name = ?1",
                [table_name],
                |row| row.get::<_, i64>(0),
            )
            .unwrap()
            > 0
    }

    fn has_index(&self, index_name: &str) -> bool {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?1",
                [index_name],
                |row| row.get::<_, i64>(0),
            )
            .unwrap()
            > 0
    }
}

#[test]
fn migrations_create_core_tables_and_fts() {
    let db = TestDb::new();
    db.run_migrations();

    assert!(db.has_table("workspaces"));
    assert!(db.has_table("conversations"));
    assert!(db.has_table("messages"));
    assert!(db.has_table("notes"));
    assert!(db.has_table("sync_runs"));
    assert!(db.has_table("conversation_search"));
    assert!(db.has_table("note_search"));
}

#[test]
fn migrations_create_archive_performance_indexes() {
    let db = TestDb::new();
    db.run_migrations();

    assert!(db.has_index("idx_conversations_updated_at"));
    assert!(db.has_index("idx_messages_conversation_created_at"));
    assert!(db.has_index("idx_messages_conversation_type_created_at"));
    assert!(db.has_index("idx_note_sources_conversation_id"));
}
