use keyring::Entry;

pub fn store_api_key(api_key: &str) -> keyring::Result<()> {
    Entry::new("context-vault", "llm-api-key")?.set_password(api_key)
}

pub fn load_api_key() -> keyring::Result<String> {
    Entry::new("context-vault", "llm-api-key")?.get_password()
}
