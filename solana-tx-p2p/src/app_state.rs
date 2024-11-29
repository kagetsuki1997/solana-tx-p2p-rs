use axum::extract::FromRef;
use tokio::sync::mpsc;

use crate::service::{DefaultPeerService, PeerService, PeerWorkerInboundEvent};

pub trait AppState: Clone + Send + Sync + 'static {
    type PeerService: PeerService + FromRef<Self> + Send + Sync + Clone;

    fn peer_service(&self) -> Self::PeerService;
}

#[derive(Clone, FromRef)]
pub struct DefaultAppState {
    peer_service: DefaultPeerService,
}

impl DefaultAppState {
    #[must_use]
    pub const fn new(peer_worker_inbound_sender: mpsc::Sender<PeerWorkerInboundEvent>) -> Self {
        Self { peer_service: DefaultPeerService::new(peer_worker_inbound_sender) }
    }
}

impl AppState for DefaultAppState {
    type PeerService = DefaultPeerService;

    fn peer_service(&self) -> Self::PeerService { self.peer_service.clone() }
}
