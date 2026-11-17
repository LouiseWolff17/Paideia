use std::{net::SocketAddr, path::PathBuf};

/// Local server configuration, read from environment variables (and `.env`).
pub struct Config {
    pub bind: SocketAddr,
    pub data_dir: PathBuf,
    pub web_dist: PathBuf,
    /// Kept private to the server. Never serialised, logged or sent to the browser.
    anthropic_api_key: Option<String>,
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

        let anthropic_api_key = std::env::var("ANTHROPIC_API_KEY")
            .ok()
            .map(|k| k.trim().to_owned())
            .filter(|k| !k.is_empty());

        Ok(Self { bind, data_dir, web_dist, anthropic_api_key })
    }

    /// Defaults for tests: no AI key, nothing read from the environment.
    #[cfg(test)]
    pub fn for_tests() -> Self {
        Self {
            bind: "127.0.0.1:0".parse().unwrap(),
            data_dir: PathBuf::from("./unused-in-tests"),
            web_dist: PathBuf::from("./unused-in-tests"),
            anthropic_api_key: None,
        }
    }

    pub fn ai_configured(&self) -> bool {
        self.anthropic_api_key.is_some()
    }

    /// For the AI adapter (Milestone 3). Not used yet.
    #[allow(dead_code)]
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
            .finish()
    }
}
