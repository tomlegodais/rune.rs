use std::net::SocketAddr;

const DEFAULT_VERSION: u32 = 530;

#[derive(Debug, Clone)]
pub struct Js5Config {
    pub bind_addr: SocketAddr,
    pub version: u32,
    pub max_connections: usize,
    pub request_buffer_size: usize,
}

impl Default for Js5Config {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:43594".parse().unwrap(),
            version: DEFAULT_VERSION,
            max_connections: 100,
            request_buffer_size: 1024,
        }
    }
}
