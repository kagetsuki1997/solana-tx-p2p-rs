tonic::include_proto!("p2p");

pub mod v1 {
    pub use self::peer_service_server::{PeerService, PeerServiceServer};

    tonic::include_proto!("p2p.v1");
}
