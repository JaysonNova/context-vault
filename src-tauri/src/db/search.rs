pub fn build_like_pattern(text: Option<String>) -> Option<String> {
    text.map(|value| format!("%{value}%"))
}
