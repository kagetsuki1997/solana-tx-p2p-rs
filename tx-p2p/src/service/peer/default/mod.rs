use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use snafu::{OptionExt, ResultExt};

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
    async fn discovery(&self) -> Result<Vec<model::Peer>> { todo!() }
}
