pub mod error;
mod peer;

use axum::async_trait;

pub use self::{
    error::Result,
    peer::{DefaultPeerService, PeerWorker},
};
use crate::model;

#[async_trait]
pub trait PeerService {
    async fn discovery(&self) -> Result<Vec<model::Peer>>;
}
