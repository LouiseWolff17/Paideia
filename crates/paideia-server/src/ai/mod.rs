//! The AI tutor, kept behind a small adapter (design doc §6).
//!
//! The rest of the server asks for a [`TutorContent`] given a [`Prompt`]; it
//! never sees provider-specific request formats. Every reply is validated with
//! [`TutorContent::validated`] before it is returned.
//!
//! Nothing in this module logs prompts, material or replies.

mod anthropic;
pub mod prompt;

use paideia_core::TutorContent;

pub use anthropic::AnthropicTutor;
pub use prompt::Prompt;

use crate::config::Config;

/// A validated reply, and which model produced it.
#[derive(Debug)]
pub struct Generated {
    pub content: TutorContent,
    pub model: String,
}

/// Why the tutor could not produce a reply. Each maps to a calm message in
/// `error.rs`; details are logged without any learner content.
#[derive(Debug)]
pub enum AiError {
    /// No API key is configured.
    NotConfigured,
    /// The key was not accepted (401/403).
    KeyRejected,
    /// Rate limited or overloaded (429/529).
    Busy,
    /// Network problem, timeout or server error.
    Unavailable(String),
    /// The model declined the request.
    Declined,
    /// The reply was missing, cut off, not valid JSON, or failed validation.
    BadReply(String),
    /// The API rejected our request (a bug on our side, e.g. 400).
    Request(String),
}

/// Which tutor backs the server.
pub enum Tutor {
    /// No API key: every non-AI feature still works.
    Off,
    Anthropic(AnthropicTutor),
    #[cfg(test)]
    Fake(fake::FakeTutor),
}

impl Tutor {
    pub fn from_config(config: &Config) -> Self {
        match config.anthropic_api_key() {
            Some(key) => Tutor::Anthropic(AnthropicTutor::new(
                key.to_owned(),
                config.anthropic_model.clone(),
                config.anthropic_effort.clone(),
            )),
            None => Tutor::Off,
        }
    }

    pub fn is_configured(&self) -> bool {
        !matches!(self, Tutor::Off)
    }

    pub async fn generate(&self, prompt: &Prompt) -> Result<Generated, AiError> {
        match self {
            Tutor::Off => Err(AiError::NotConfigured),
            Tutor::Anthropic(t) => t.generate(prompt).await,
            #[cfg(test)]
            Tutor::Fake(t) => t.generate(prompt),
        }
    }
}

/// Parse and validate the JSON text of a reply. Shared by every provider.
fn parse_content(json: &str) -> Result<TutorContent, AiError> {
    let raw: TutorContent = serde_json::from_str(json)
        .map_err(|e| AiError::BadReply(format!("reply is not the expected JSON: {e}")))?;
    raw.validated().map_err(|e| AiError::BadReply(e.to_string()))
}

#[cfg(test)]
pub mod fake {
    //! A scripted tutor for tests. It records every prompt it receives.

    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use super::{parse_content, AiError, Generated, Prompt};

    #[derive(Clone, Default)]
    pub struct FakeTutor {
        replies: Arc<Mutex<VecDeque<Result<String, AiError>>>>,
        pub prompts: Arc<Mutex<Vec<Prompt>>>,
    }

    impl FakeTutor {
        /// Queue a raw JSON reply, exactly as a provider would return it.
        pub fn reply(&self, json: serde_json::Value) {
            self.replies.lock().unwrap().push_back(Ok(json.to_string()));
        }

        pub fn fail(&self, error: AiError) {
            self.replies.lock().unwrap().push_back(Err(error));
        }

        pub fn generate(&self, prompt: &Prompt) -> Result<Generated, AiError> {
            self.prompts.lock().unwrap().push(prompt.clone());
            let next = self.replies.lock().unwrap().pop_front().expect("no fake reply queued");
            Ok(Generated { content: parse_content(&next?)?, model: "fake-tutor".into() })
        }
    }
}
