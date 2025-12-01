use crate::account::Account;
use crate::config::GameConfig;
use crate::world::World;
use async_trait::async_trait;
use net::{LoginOutcome, LoginRequest, LoginService, LoginSuccess, SessionError};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct WorldLoginService {
    config: GameConfig,
    world: Arc<Mutex<World>>,
}

impl WorldLoginService {
    pub fn new(config: GameConfig, world: Arc<Mutex<World>>) -> Self {
        Self { config, world }
    }

    fn validate_session(
        &self,
        session_key: i64,
        server_key: i64,
        version: u32,
        _crc: &[u32; 31],
    ) -> Option<LoginOutcome> {
        if server_key != session_key {
            return Some(LoginOutcome::BadSessionId);
        }

        if version != self.config.client_version {
            return Some(LoginOutcome::GameUpdated);
        }

        None
    }

    async fn load_account_by_username(&self, username: &str) -> Result<Account, SessionError> {
        // TODO: Accounts Service (SQL, Memory, Disk, etc...)

        Ok(Account {
            id: 1,
            username: username.to_string(),
            _password_hash: "fake-hash".to_string(),
            rights: 0,
        })
    }

    async fn verify_password(&self, _account: &Account, _password: &str) -> Option<LoginOutcome> {
        None
    }
}

#[async_trait]
impl LoginService for WorldLoginService {
    async fn authenticate(
        &self,
        req: LoginRequest,
        session_key: i64,
    ) -> Result<LoginOutcome, SessionError> {
        if let Some(outcome) =
            self.validate_session(session_key, req.server_key, req.version, &req.crc)
        {
            return Ok(outcome);
        }

        let account = match self.load_account_by_username(&req.username).await {
            Ok(account) => account,
            Err(_) => return Ok(LoginOutcome::InvalidCredentials),
        };

        if let Some(outcome) = self.verify_password(&account, &req.password).await {
            return Ok(outcome);
        }

        let (player_index, inbox_tx, outbound_rx) = {
            let mut world = self.world.lock().await;
            let (player_index, inbox_tx, outbound_rx) = world.register_player(&account, req.display_mode);

            world.on_player_login(player_index).await;
            (player_index, inbox_tx, outbound_rx)
        };

        let success = LoginSuccess {
            rights: account.rights,
            player_index,
            members: true,
            inbox_tx,
            outbound_rx,
        };

        Ok(LoginOutcome::Success(success))
    }
}
