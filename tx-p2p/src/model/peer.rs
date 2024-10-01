use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::proto::peer as proto;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Peer {
    pub id: String,
    pub is_signer: bool,
    pub is_leader: bool,
}

impl From<proto::Peer> for Peer {
    fn from(proto::Peer { id, is_signer, is_leader }: proto::Peer) -> Self {
        Self { id, is_signer, is_leader }
    }
}

impl From<Peer> for proto::Peer {
    fn from(Peer { id, is_signer, is_leader }: Peer) -> Self { Self { id, is_signer, is_leader } }
}
