use crate::LoginOutcome;
use crate::error::SessionError;
use crate::message::LoginRequest;
use async_trait::async_trait;

#[async_trait]
pub trait LoginService: Send + Sync {
    async fn authenticate(
        &self,
        req: LoginRequest,
        session_key: i64,
    ) -> Result<LoginOutcome, SessionError>;

    async fn logout(&self, player_index: usize);
}
