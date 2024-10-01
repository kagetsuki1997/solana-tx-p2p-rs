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
    async fn discovery_peers(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto::v1::Peers>, Status> {
        let peers = self.inner.discovery_peers().await?;

        Ok(Response::new(proto::v1::Peers { peers }))
    }

    async fn list_signed_messages(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto::Transactions>, Status> {
        let signed_messages = self.inner.list_signed_messages().await?;

        Ok(Response::new(proto::Transactions {
            transactions: signed_messages.into_iter().map(Into::into).collect(),
        }))
    }

    async fn list_relayed_transactions(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto::v1::RelayTransactions>, Status> {
        let signatures = self.inner.list_relayed_transactions().await?;

        Ok(Response::new(proto::v1::RelayTransactions { signatures }))
    }
}
