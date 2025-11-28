use crate::error::SessionError;
use tokio::net::TcpStream;

pub struct LoginHandler;

impl LoginHandler {
    pub async fn run(_stream: TcpStream) -> anyhow::Result<(), SessionError> {
        unimplemented!()
    }
}
