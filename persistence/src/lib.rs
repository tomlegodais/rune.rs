pub mod account;
mod migration;
pub mod player;

mod config;
mod database;

pub use config::DatabaseConfig;
pub use database::{connect, PersistenceModule};
pub use shaku;