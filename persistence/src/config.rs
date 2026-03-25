use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://runers:p@ssw0rd!@localhost:5432/runers".to_string(),
            max_connections: 5,
        }
    }
}
