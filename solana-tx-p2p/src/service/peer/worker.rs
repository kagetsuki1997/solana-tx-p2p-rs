use std::{collections::HashSet, sync::Arc, time::Duration};

use lazy_static::lazy_static;
use libp2p::{
    floodsub::{Floodsub, FloodsubEvent, FloodsubMessage, Topic},
    futures::StreamExt,
    identity,
    mdns::{tokio::Behaviour as Mdns, Config as MdnsConfig, Event as MdnsEvent},
    noise::Config as NoiseConfig,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    tcp::Config as TcpConfig,
    yamux::Config as YamuxConfig,
    PeerId, SwarmBuilder,
};
use snafu::ResultExt;
use solana_sdk::{signature::Keypair as SolanaKeypair, transaction::Transaction};
use tokio::sync::{mpsc, oneshot, RwLock};

use crate::{
    service::{
        error,
        round_robin_election::{ElectionWorkerInboundEvent, LeaderSyncInfo},
        RelayerInboundEvent, Result, SignerInboundEvent,
    },
    ShutdownSignal,
};

lazy_static! {
    static ref MESSAGE_TOPIC: Topic = Topic::new("message");
    static ref HEARTBEAT_TOPIC: Topic = Topic::new("heartbeat");
    static ref RELAYER_INFO_TOPIC: Topic = Topic::new("relayer-info");
    static ref SIGNER_INFO_TOPIC: Topic = Topic::new("signer-info");
    static ref TRANSACTION_TOPIC: Topic = Topic::new("transaction");
    static ref RELAYED_TRANSACTION_TOPIC: Topic = Topic::new("relayed-transaction");
    static ref TOPICS: Vec<Topic> = vec![
        MESSAGE_TOPIC.clone(),
        HEARTBEAT_TOPIC.clone(),
        RELAYER_INFO_TOPIC.clone(),
        SIGNER_INFO_TOPIC.clone(),
        TRANSACTION_TOPIC.clone(),
        RELAYED_TRANSACTION_TOPIC.clone(),
    ];
}

enum Action {
    Input(Option<String>),
    InboundEvent(Option<PeerWorkerInboundEvent>),
    Swarm(SwarmEvent<PeerBehaviourEvent>),
    Stop,
}

#[derive(Debug)]
pub enum PeerWorkerInboundEvent {
    MessageTrigger,
    HeartbeatTrigger,
    RelayerSyncInfo(LeaderSyncInfo),
    SignerSyncInfo(LeaderSyncInfo),
    Transaction(Transaction),
    RelayedTransaction(String),
    Instruction(PeerWorkerInstruction),
}

#[derive(Debug)]
pub enum PeerWorkerInstruction {
    ListPeers(oneshot::Sender<Vec<String>>),
    ListSignedMessages(oneshot::Sender<Vec<Transaction>>),
    ListRelayedTransactions(oneshot::Sender<Vec<String>>),
}

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "PeerBehaviourEvent")]
struct PeerBehaviour {
    floodsub: Floodsub,
    mdns: Mdns,
}

#[derive(Debug)]
enum PeerBehaviourEvent {
    Floobsub(FloodsubEvent),
    Mdns(MdnsEvent),
}

impl From<FloodsubEvent> for PeerBehaviourEvent {
    fn from(event: FloodsubEvent) -> Self { Self::Floobsub(event) }
}

impl From<MdnsEvent> for PeerBehaviourEvent {
    fn from(event: MdnsEvent) -> Self { Self::Mdns(event) }
}

#[derive(Debug)]
pub struct PeerWorker {
    key: identity::Keypair,
    peer_id: PeerId,
    peers: Arc<RwLock<Vec<String>>>,
    relayer: Arc<RwLock<String>>,
    signer: Arc<RwLock<String>>,

    relayer_heartbeat_sender: mpsc::Sender<()>,
    signer_heartbeat_sender: mpsc::Sender<()>,

    relayer_election_worker_inbound_sender: mpsc::Sender<ElectionWorkerInboundEvent>,
    signer_election_worker_inbound_sender: mpsc::Sender<ElectionWorkerInboundEvent>,

    solana_relayer_inbound_sender: mpsc::Sender<RelayerInboundEvent>,
    solana_signer_inbound_sender: mpsc::Sender<SignerInboundEvent>,

    peer_worker_inbound_sender: mpsc::Sender<PeerWorkerInboundEvent>,
    peer_worker_inbound_receiver: mpsc::Receiver<PeerWorkerInboundEvent>,

    signed_messages: Arc<RwLock<Vec<Transaction>>>,
    relayed_transactions: Arc<RwLock<Vec<String>>>,
}

impl PeerWorker {
    #[must_use]
    pub fn new(
        peers: Arc<RwLock<Vec<String>>>,
        relayer: Arc<RwLock<String>>,
        signer: Arc<RwLock<String>>,
        relayer_heartbeat_sender: mpsc::Sender<()>,
        signer_heartbeat_sender: mpsc::Sender<()>,
        relayer_election_worker_inbound_sender: mpsc::Sender<ElectionWorkerInboundEvent>,
        signer_election_worker_inbound_sender: mpsc::Sender<ElectionWorkerInboundEvent>,
        solana_relayer_inbound_sender: mpsc::Sender<RelayerInboundEvent>,
        solana_signer_inbound_sender: mpsc::Sender<SignerInboundEvent>,
    ) -> Self {
        let key = identity::Keypair::generate_ed25519();
        let peer_id = key.public().into();

        let (peer_worker_inbound_sender, peer_worker_inbound_receiver) = mpsc::channel(100);

        Self {
            key,
            peer_id,
            peers,
            relayer,
            signer,
            relayer_heartbeat_sender,
            signer_heartbeat_sender,
            relayer_election_worker_inbound_sender,
            signer_election_worker_inbound_sender,
            solana_relayer_inbound_sender,
            solana_signer_inbound_sender,
            peer_worker_inbound_receiver,
            peer_worker_inbound_sender,
            signed_messages: Arc::new(RwLock::new(Vec::new())),
            relayed_transactions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    #[must_use]
    pub fn peer_id(&self) -> PeerId { self.peer_id.clone() }

    #[must_use]
    pub fn solana_keypair(&self) -> SolanaKeypair {
        let ed25519_key = self.key.clone().try_into_ed25519().expect("must be ed25519 keypair");
        solana_sdk::signature::Keypair::from_bytes(&ed25519_key.to_bytes())
            .expect("must be valid ed25519 keypair")
    }

    #[must_use]
    pub fn peer_worker_inbound_sender(&self) -> mpsc::Sender<PeerWorkerInboundEvent> {
        self.peer_worker_inbound_sender.clone()
    }

    /// The main worker to handle:
    /// - send messages to the p2p network
    /// - send peer heartbeat to the p2p network
    /// - send relayer sync info to the p2p network
    /// - send signer sync info to the p2p network
    /// - handle messages from the p2p network and output to other workers
    /// - handle stdin
    ///
    /// # Errors
    ///
    /// return error when fail to create swarm
    pub async fn start(
        mut self,
        mut shutdown_signal: ShutdownSignal,
        mut stdin_receiver: mpsc::Receiver<String>,
    ) -> Result<()> {
        tracing::info!("Start Peer:  peer_id={}", self.peer_id,);

        {
            self.peers.write().await.push(self.peer_id.to_string());
        }

        let mut swarm = start_swarm(self.peer_id.clone(), &*TOPICS, self.key.clone()).await?;

        loop {
            let action = {
                tokio::select! {
                    event = self.peer_worker_inbound_receiver.recv() => Action::InboundEvent(event),
                    line = stdin_receiver.recv() => Action::Input(line),
                    event = swarm.select_next_some() => Action::Swarm(event),
                    () = shutdown_signal.wait() => Action::Stop,
                }
            };

            match action {
                Action::Input(line) => {
                    if let Some(line) = line {
                        match line.as_str() {
                            cmd if cmd.starts_with("ls p") => handle_list_peers(&mut swarm).await,
                            cmd if cmd.starts_with("ls s") => tracing::info!(
                                "Signed Messages: {:?}",
                                *self.signed_messages.read().await
                            ),
                            cmd if cmd.starts_with("ls t") => tracing::info!(
                                "Relayed Transactions: {:?}",
                                *self.relayed_transactions.read().await
                            ),
                            _ => tracing::error!("unknown command from stdin"),
                        }
                    } else {
                        tracing::warn!("stdin_sender is closed");
                    }
                }
                Action::InboundEvent(event) => match event {
                    None => {
                        tracing::warn!("`peer_worker_inbound_receiver` is closed");
                        break;
                    }
                    Some(PeerWorkerInboundEvent::MessageTrigger) => {
                        tracing::warn!("Message trigger");
                        // send message to p2p network
                        let message = format!("Message from {}", self.peer_id.clone());
                        swarm
                            .behaviour_mut()
                            .floodsub
                            .publish(MESSAGE_TOPIC.clone(), message.clone());

                        if let Err(err) = self
                            .solana_signer_inbound_sender
                            .send(SignerInboundEvent::RawMessage(message.into()))
                            .await
                        {
                            tracing::error!("Fail to send message to solana signer: {err}");
                            break;
                        }
                    }
                    Some(PeerWorkerInboundEvent::HeartbeatTrigger) => {
                        self.heartbeat_trigger(&mut swarm, HEARTBEAT_TOPIC.clone()).await;
                    }
                    Some(PeerWorkerInboundEvent::RelayerSyncInfo(leader_sync_info)) => {
                        // send relayer info to p2p network
                        swarm.behaviour_mut().floodsub.publish(
                            RELAYER_INFO_TOPIC.clone(),
                            serde_json::to_vec(&leader_sync_info)
                                .expect("LeaderSyncInfo is valid json"),
                        );
                    }
                    Some(PeerWorkerInboundEvent::SignerSyncInfo(leader_sync_info)) => {
                        // send signer info to p2p network
                        swarm.behaviour_mut().floodsub.publish(
                            SIGNER_INFO_TOPIC.clone(),
                            serde_json::to_vec(&leader_sync_info)
                                .expect("LeaderSyncInfo is valid json"),
                        );
                    }
                    Some(PeerWorkerInboundEvent::Transaction(transaction)) => {
                        // send transaction to p2p network
                        tracing::info!("Transaction: {transaction:?}");
                        swarm.behaviour_mut().floodsub.publish(
                            TRANSACTION_TOPIC.clone(),
                            serde_json::to_vec(&transaction).expect("Transaction is valid json"),
                        );

                        if let Err(err) = self
                            .solana_relayer_inbound_sender
                            .send(RelayerInboundEvent::Transaction(transaction.clone()))
                            .await
                        {
                            tracing::error!("Fail to send transaction to solana relayer: {err}");
                            break;
                        }

                        self.signed_messages.write().await.push(transaction);
                    }
                    Some(PeerWorkerInboundEvent::RelayedTransaction(transaction)) => {
                        self.relayed_transactions.write().await.push(transaction);
                    }
                    Some(PeerWorkerInboundEvent::Instruction(instruction)) => match instruction {
                        PeerWorkerInstruction::ListPeers(sender) => {
                            let peers = self.peers.read().await.clone();
                            drop(sender.send(peers));
                        }
                        PeerWorkerInstruction::ListSignedMessages(sender) => {
                            let signed_messages = self.signed_messages.read().await.clone();
                            drop(sender.send(signed_messages));
                        }
                        PeerWorkerInstruction::ListRelayedTransactions(sender) => {
                            let relayed_transactions =
                                self.relayed_transactions.read().await.clone();
                            drop(sender.send(relayed_transactions));
                        }
                    },
                },
                Action::Swarm(swarm_event) => match swarm_event {
                    SwarmEvent::Behaviour(PeerBehaviourEvent::Floobsub(
                        FloodsubEvent::Message(msg),
                    )) => {
                        if let Err(()) = self.handle_message(&msg).await {
                            break;
                        }
                    }
                    SwarmEvent::Behaviour(PeerBehaviourEvent::Mdns(mdns_event)) => match mdns_event
                    {
                        MdnsEvent::Discovered(discovered_list) => {
                            for (peer, _addr) in discovered_list {
                                swarm.behaviour_mut().floodsub.add_node_to_partial_view(peer);
                            }
                        }
                        MdnsEvent::Expired(expired_list) => {
                            for (peer, _addr) in expired_list {
                                if swarm
                                    .behaviour_mut()
                                    .mdns
                                    .discovered_nodes()
                                    .into_iter()
                                    .find(|&node| *node == peer)
                                    .is_none()
                                {
                                    swarm
                                        .behaviour_mut()
                                        .floodsub
                                        .remove_node_from_partial_view(&peer);
                                }
                            }
                        }
                    },
                    ref event @ SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        tracing::info!("New Peer connection established {event:?}");
                        {
                            self.peers.write().await.push(peer_id.to_string());
                        }
                    }
                    ref event @ SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        tracing::info!("Peer connection closed {event:?}");
                        {
                            let mut peers = self.peers.write().await;
                            if let Ok(idx) = peers.binary_search(&peer_id.to_string()) {
                                peers.remove(idx);
                            }
                        }
                    }
                    _ => {
                        tracing::debug!("Unhandled event {swarm_event:?}")
                    }
                },
                Action::Stop => break,
            }
        }

        tracing::warn!("PeerWorker is stopped");

        Ok(())
    }

    async fn heartbeat_trigger(
        &mut self,
        swarm: &mut Swarm<PeerBehaviour>,
        heartbeat_topic: Topic,
    ) {
        tracing::debug!("Heartbeat trigger {}", self.peer_id.clone());
        let message = format!("Heartbeat from {}", self.peer_id.clone());
        swarm.behaviour_mut().floodsub.publish(heartbeat_topic, message);

        // send heartbeat to election worker
        self.send_leader_heartbeat(&self.peer_id).await;
    }

    async fn send_leader_heartbeat(&self, source: &PeerId) {
        // send heartbeat to relayer election worker if source is relayer
        if source.to_string() == *self.relayer.read().await {
            let _ = self.relayer_heartbeat_sender.send(()).await;
        }

        // send heartbeat to signer election worker if source is signer
        if source.to_string() == *self.signer.read().await {
            let _ = self.signer_heartbeat_sender.send(()).await;
        }
    }

    async fn handle_message(&self, msg: &FloodsubMessage) -> Result<(), ()> {
        if msg.topics.contains(&MESSAGE_TOPIC) {
            let message = String::from_utf8_lossy(&msg.data);
            tracing::warn!("Receive message from {:?}: {message}", msg.source);
            if let Err(err) = self
                .solana_signer_inbound_sender
                .send(SignerInboundEvent::RawMessage(msg.data.clone()))
                .await
            {
                tracing::error!("Fail to send message to solana signer: {err}");
                return Err(());
            }
        } else if msg.topics.contains(&HEARTBEAT_TOPIC) {
            // send heartbeat to election worker
            self.send_leader_heartbeat(&msg.source).await;
        } else if msg.topics.contains(&RELAYER_INFO_TOPIC) {
            let Ok(leader_sync_info) = serde_json::from_slice::<LeaderSyncInfo>(&msg.data) else {
                tracing::error!("Invalid leader sync info");
                return Ok(());
            };

            if let Err(err) = self
                .relayer_election_worker_inbound_sender
                .send(ElectionWorkerInboundEvent::LeaderSyncInfo(leader_sync_info))
                .await
            {
                tracing::error!("Fail to send relayer leader info to election worker: {err}");
                return Err(());
            }
        } else if msg.topics.contains(&SIGNER_INFO_TOPIC) {
            let Ok(leader_sync_info) = serde_json::from_slice::<LeaderSyncInfo>(&msg.data) else {
                tracing::error!("Invalid leader sync info");
                return Ok(());
            };

            if let Err(err) = self
                .signer_election_worker_inbound_sender
                .send(ElectionWorkerInboundEvent::LeaderSyncInfo(leader_sync_info))
                .await
            {
                tracing::error!("Fail to send signer leader info to election worker: {err}");
                return Err(());
            }
        } else if msg.topics.contains(&RELAYED_TRANSACTION_TOPIC) {
            let transaction_signature = String::from_utf8_lossy(&msg.data);

            self.relayed_transactions.write().await.push(transaction_signature.to_string());
        }

        Ok(())
    }
}

async fn handle_list_peers(swarm: &mut Swarm<PeerBehaviour>) {
    tracing::info!("Discovered Peers:");
    let nodes = swarm.behaviour().mdns.discovered_nodes();
    let mut unique_peers = HashSet::new();
    for peer in nodes {
        unique_peers.insert(peer);
    }
    unique_peers.iter().for_each(|p| tracing::info!("{}", p));
}

async fn start_swarm(
    peer_id: PeerId,
    topics: &[Topic],
    key: identity::Keypair,
) -> Result<Swarm<PeerBehaviour>> {
    let mut behaviour = PeerBehaviour {
        floodsub: Floodsub::new(peer_id.clone()),
        mdns: Mdns::new(
            MdnsConfig {
                ttl: Duration::from_secs(60),
                query_interval: Duration::from_secs(30),
                enable_ipv6: false,
            },
            peer_id,
        )
        .expect("must create mdns"),
    };

    for topic in topics {
        behaviour.floodsub.subscribe(topic.clone());
    }

    let mut swarm = SwarmBuilder::with_existing_identity(key)
        .with_tokio()
        .with_tcp(TcpConfig::default(), NoiseConfig::new, YamuxConfig::default)
        .context(error::SwarmWithTcpSnafu)?
        .with_behaviour(|_key| behaviour)
        .expect("swarm with behaviour is infallible")
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse().expect("can get a local socket"))
        .expect("swarm can be started");

    Ok(swarm)
}
