use std::{net::SocketAddr, path::PathBuf};

/// Local server configuration, read from environment variables (and `.env`).
pub struct Config {
    pub bind: SocketAddr,
    pub data_dir: PathBuf,
    pub web_dist: PathBuf,
    /// Kept private to the server. Never serialised, logged or sent to the browser.
    anthropic_api_key: Option<String>,
    /// Claude model used by the tutor.
    pub anthropic_model: String,
    /// How much effort the model spends thinking: low, medium, high, xhigh or max.
    pub anthropic_effort: String,
}

pub const DEFAULT_MODEL: &str = "claude-opus-5-5";
pub const DEFAULT_EFFORT: &str = "medium";
const EFFORTS: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];

fn non_empty_var(name: &str) -> Option<String> {
    std::env::var(name).ok().map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let bind: SocketAddr = std::env::var("PAIDEIA_BIND")
            .unwrap_or_else(|_| "127.0.0.1:3000".into())
            .parse()
            .map_err(|e| format!("PAIDEIA_BIND is not a valid address: {e}"))?;

        let data_dir = std::env::var("PAIDEIA_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./data"));

        let web_dist = std::env::var("PAIDEIA_WEB_DIST")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./crates/paideia-web/dist"));

        let anthropic_api_key = non_empty_var("ANTHROPIC_API_KEY");
        let anthropic_model = non_empty_var("ANTHROPIC_MODEL").unwrap_or_else(|| DEFAULT_MODEL.into());
        let anthropic_effort = non_empty_var("ANTHROPIC_EFFORT").unwrap_or_else(|| DEFAULT_EFFORT.into());
        if !EFFORTS.contains(&anthropic_effort.as_str()) {
            return Err(format!(
                "ANTHROPIC_EFFORT must be one of {}, not {anthropic_effort:?}",
                EFFORTS.join(", ")
            ));
        }

        Ok(Self { bind, data_dir, web_dist, anthropic_api_key, anthropic_model, anthropic_effort })
    }

    /// Defaults for tests: no AI key, nothing read from the environment.
    #[cfg(test)]
    pub fn for_tests() -> Self {
        Self {
            bind: "127.0.0.1:0".parse().unwrap(),
            data_dir: PathBuf::from("./unused-in-tests"),
            web_dist: PathBuf::from("./unused-in-tests"),
            anthropic_api_key: None,
            anthropic_model: DEFAULT_MODEL.into(),
            anthropic_effort: DEFAULT_EFFORT.into(),
        }
    }

    /// For the AI adapter only.
    pub fn anthropic_api_key(&self) -> Option<&str> {
        self.anthropic_api_key.as_deref()
    }
}

// A hand-written Debug so the key can never end up in logs by accident.
impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("bind", &self.bind)
            .field("data_dir", &self.data_dir)
            .field("web_dist", &self.web_dist)
            .field("anthropic_api_key", &self.anthropic_api_key.as_ref().map(|_| "<set>"))
            .field("anthropic_model", &self.anthropic_model)
            .field("anthropic_effort", &self.anthropic_effort)
            .finish()
    }
}
