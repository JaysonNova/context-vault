use crate::settings::store::AppSettings;

#[tauri::command]
pub fn load_settings_command() -> AppSettings {
    AppSettings::default()
}
