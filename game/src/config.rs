use net::config::TcpConfig;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct LogConfig {
    pub level: String,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub log: LogConfig,

    #[serde(default)]
    pub tcp: TcpConfig,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "INFO".to_string(),
        }
    }
}
