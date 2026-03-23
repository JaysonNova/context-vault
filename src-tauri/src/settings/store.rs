use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppSettings {
    pub provider: String,
    pub model: String,
    pub base_url: Option<String>,
}
