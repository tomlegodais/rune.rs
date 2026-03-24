use super::entity::{self, Column, Entity as AccountEntity};
use super::{Account, Rights};
use async_trait::async_trait;
use sea_orm::prelude::Expr;
use sea_orm::*;
use shaku::{Component, Interface};

impl TryFrom<entity::Model> for Account {
    type Error = DbErr;

    fn try_from(model: entity::Model) -> Result<Self, Self::Error> {
        let rights =
            Rights::try_from(model.rights as u8).map_err(|e| DbErr::Type(e.to_string()))?;

        Ok(Account {
            id: model.id,
            username: model.username,
            password_hash: model.password_hash,
            rights,
            disabled: model.disabled,
        })
    }
}

#[async_trait]
pub trait AccountRepository: Interface {
    async fn find_by_username(&self, username: &str) -> Result<Option<Account>, DbErr>;
    async fn update_last_login(&self, account_id: i64) -> Result<(), DbErr>;
}

#[derive(Component)]
#[shaku(interface = AccountRepository)]
pub struct PgAccountRepository {
    #[shaku(default)]
    db: DatabaseConnection,
}

#[async_trait]
impl AccountRepository for PgAccountRepository {
    async fn find_by_username(&self, username: &str) -> Result<Option<Account>, DbErr> {
        AccountEntity::find()
            .filter(Column::Username.eq(username))
            .one(&self.db)
            .await?
            .map(Account::try_from)
            .transpose()
    }

    async fn update_last_login(&self, account_id: i64) -> Result<(), DbErr> {
        AccountEntity::update_many()
            .col_expr(Column::LastLogin, Expr::current_timestamp().into())
            .filter(Column::Id.eq(account_id))
            .exec(&self.db)
            .await?;

        Ok(())
    }
}
