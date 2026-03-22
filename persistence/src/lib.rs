pub mod account;
mod config;
mod database;
mod migration;
pub mod player;

pub use account::Rights;
pub use config::DatabaseConfig;
pub use database::{connect, PersistenceModule, PersistenceModuleInterface};
pub use shaku;
