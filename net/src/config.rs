use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct TcpConfig {
    pub bind_addr: SocketAddr,
    pub max_connections: usize,
    pub request_buffer_size: usize,
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:43594".parse().unwrap(),
            max_connections: 100,
            request_buffer_size: 1024,
        }
    }
}
