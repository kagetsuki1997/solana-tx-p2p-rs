use axum::extract::FromRef;

use crate::service::{DefaultPeerService, PeerService};

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
    pub fn new() -> Self { Self { peer_service: DefaultPeerService::new() } }
}

impl AppState for DefaultAppState {
    type PeerService = DefaultPeerService;

    fn peer_service(&self) -> Self::PeerService { self.peer_service.clone() }
}
