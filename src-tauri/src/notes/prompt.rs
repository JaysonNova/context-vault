pub const PROMPT_VERSION: &str = "technical_note_v1";

pub fn build_prompt(title: &str, messages: &[String]) -> String {
    let timeline = messages.join("\n\n");
    format!(
        "Generate a concise technical note.\nTitle: {title}\n\nConversation:\n{timeline}"
    )
}
