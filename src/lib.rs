pub mod config;
pub mod db;
pub mod error;
pub mod events;
pub mod graphql;
pub mod handlers;
pub mod keto;
pub mod middleware;
pub mod models;
pub mod observability;
pub mod pactum_client;
pub mod platform_events;
pub mod types;

pub use error::{AdeptusError, AdeptusResult};
pub use models::*;
pub use platform_events::*;
pub use types::*;

use crate::db::{DatabaseManager, RepositoryManager};
use crate::events::EventPublisher;
use crate::keto::KetoClient;
use crate::pactum_client::PactumClient;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseManager,
    pub repos: RepositoryManager,
    pub config: config::AppConfig,
    pub events: EventPublisher,
    pub keto: KetoClient,
    pub pactum: PactumClient,
}
