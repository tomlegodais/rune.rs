use std::sync::Arc;

use async_trait::async_trait;
use net::{LoginOutcome, LoginRequest, LoginService, LoginSuccess, SessionError};
use persistence::{
    account::AccountRepository,
    player::{PlayerData, PlayerRepository},
};
use shaku::{Component, Interface};

use crate::{config::GameConfig, world::World};

pub trait GameLoginService: LoginService + Interface {}

#[derive(Component)]
#[shaku(interface = GameLoginService)]
pub struct WorldLoginService {
    #[shaku(inject)]
    accounts: Arc<dyn AccountRepository>,

    #[shaku(inject)]
    players: Arc<dyn PlayerRepository>,

    #[shaku(default)]
    config: GameConfig,

    #[shaku(default)]
    world: Arc<World>,
}

impl GameLoginService for WorldLoginService {}

impl WorldLoginService {
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

    async fn load_or_create_player(&self, account_id: i64) -> Result<PlayerData, SessionError> {
        match self.players.find_by_account_id(account_id).await {
            Ok(Some(data)) => Ok(data),
            Ok(None) => self
                .players
                .create_default(account_id)
                .await
                .map_err(|e| SessionError::Internal(e.to_string())),
            Err(e) => Err(SessionError::Internal(e.to_string())),
        }
    }
}

#[async_trait]
impl LoginService for WorldLoginService {
    #[rustfmt::skip]
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

        let account = match self.accounts.find_by_username(&req.username).await {
            Ok(Some(account)) => account,
            _ => return Ok(LoginOutcome::InvalidCredentials),
        };

        if !account.verify_password(&req.password) { return Ok(LoginOutcome::InvalidCredentials); }
        if account.disabled { return Ok(LoginOutcome::AccountDisabled); }
        if self.world.is_online(account.id) { return Ok(LoginOutcome::AlreadyOnline); }

        let _ = self.accounts.update_last_login(account.id).await;
        let player_data = self.load_or_create_player(account.id).await?;
        let (player_index, inbox_tx, outbound_rx) =
            self.world.register_player(&account, &player_data, req.display_mode);

        self.world.player_mut(player_index).on_login().await;

        let success = LoginSuccess {
            rights: account.rights.into(),
            player_index,
            members: true,
            inbox_tx,
            outbound_rx,
        };

        Ok(LoginOutcome::Success(success))
    }

    async fn logout(&self, player_index: usize) {
        if let Some(data) = self.world.unregister_player(player_index)
            && let Err(e) = self.players.save(&data).await
        {
            tracing::warn!(error = %e, "Failed to Save Player Data");
        }
    }
}
