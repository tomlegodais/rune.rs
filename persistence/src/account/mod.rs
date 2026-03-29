pub(crate) mod entity;
mod repository;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use num_enum::{IntoPrimitive, TryFromPrimitive};
pub use repository::AccountRepository;
pub(crate) use repository::{PgAccountRepository, PgAccountRepositoryParameters};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum Rights {
    Standard = 0,
    Moderator = 1,
    Admin = 2,
}

#[derive(Debug)]
pub struct Account {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub rights: Rights,
    pub disabled: bool,
}

impl Account {
    pub fn display_name(&self) -> String {
        util::format_display_name(&self.username)
    }

    pub fn verify_password(&self, password: &str) -> bool {
        PasswordHash::new(&self.password_hash)
            .map(|hash| Argon2::default().verify_password(password.as_bytes(), &hash).is_ok())
            .unwrap_or(false)
    }
}
