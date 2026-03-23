fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            context_vault_lib::commands::archive::list_conversations_command,
            context_vault_lib::commands::archive::get_archive_facets_command,
            context_vault_lib::commands::archive::get_conversation_detail_command,
            context_vault_lib::commands::notes::list_notes_command,
            context_vault_lib::commands::settings::load_settings_command,
            context_vault_lib::commands::sync::list_sync_runs_command,
            context_vault_lib::commands::sync::run_sync_command
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Context Vault application");
}
