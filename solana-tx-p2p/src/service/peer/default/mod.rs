use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use snafu::{OptionExt, ResultExt};
use solana_sdk::transaction::Transaction;

use crate::{
    model,
    service::{error::Result, PeerService},
};

#[derive(Clone)]
pub struct DefaultPeerService {}

impl DefaultPeerService {
    #[inline]
    #[must_use]
    pub fn new() -> Self { Self {} }
}

#[async_trait]
impl PeerService for DefaultPeerService {
    async fn discovery_peers(&self) -> Result<Vec<String>> { todo!() }

    async fn list_signed_messages(&self) -> Result<Vec<Transaction>> { todo!() }

    async fn list_relayed_transactions(&self) -> Result<Vec<String>> { todo!() }
}
