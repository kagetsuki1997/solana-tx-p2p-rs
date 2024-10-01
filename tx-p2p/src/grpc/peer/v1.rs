use async_trait::async_trait;
use tonic::{Request, Response, Result, Status};

use crate::{
    proto::peer::{self as proto},
    service,
};

pub struct PeerService<T>
where
    T: service::PeerService + Send + Sync + 'static,
{
    inner: T,
}

impl<T> PeerService<T>
where
    T: service::PeerService + Send + Sync + 'static,
{
    pub const fn new(inner: T) -> Self { Self { inner } }
}

#[async_trait]
impl<T> proto::v1::PeerService for PeerService<T>
where
    T: service::PeerService + Send + Sync + 'static,
{
    async fn discovery(&self, _request: Request<()>) -> Result<Response<proto::Peers>, Status> {
        let peers = self.inner.discovery().await?;

        Ok(Response::new(proto::Peers { peers: peers.into_iter().map(Into::into).collect() }))
    }
}
