use net::TcpConfig;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct LogConfig {
    pub level: String,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct GameConfig {
    pub client_version: u32,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub log: LogConfig,

    #[serde(default)]
    pub game: GameConfig,

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

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            client_version: 592,
        }
    }
}
