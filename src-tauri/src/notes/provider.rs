use anyhow::Result;

pub trait NoteProvider {
    fn generate(&self, prompt: &str) -> Result<String>;
}

pub struct StubProvider {
    response: String,
}

impl StubProvider {
    pub fn success(response: &str) -> Self {
        Self {
            response: response.to_string(),
        }
    }
}

impl NoteProvider for StubProvider {
    fn generate(&self, _prompt: &str) -> Result<String> {
        Ok(self.response.clone())
    }
}
