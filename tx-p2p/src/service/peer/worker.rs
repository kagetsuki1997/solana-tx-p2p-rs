use std::{collections::HashSet, sync::Arc, time::Duration};

use libp2p::{
    core::upgrade,
    floodsub::{Floodsub, FloodsubEvent, Topic},
    futures::StreamExt,
    identity,
    mdns::{tokio::Behaviour as Mdns, Config as MdnsConfig, Event as MdnsEvent},
    noise::Config as NoiseConfig,
    swarm::{behaviour, NetworkBehaviour, Swarm, SwarmEvent},
    tcp::Config as TcpConfig,
    yamux::Config as YamuxConfig,
    PeerId, SwarmBuilder, Transport,
};
use rand::random;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use tokio::{
    io::AsyncBufReadExt,
    sync::{mpsc, RwLock},
    time,
};

use crate::{
    service::{error, Result},
    ShutdownSignal,
};

const MESSAGE_TOPIC: &str = "message";

enum Action {
    Input(Option<String>),
    MessageTrigger,
    Swarm(SwarmEvent<PeerBehaviourEvent>),
    Stop,
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
    pub message_duration: Duration,
    pub relay_leader_duration: Duration,
    pub signing_leader_duration: Duration,

    pub key: identity::Keypair,
    pub peer_id: PeerId,
    pub peers: Arc<RwLock<Vec<String>>>,
}

impl PeerWorker {
    #[must_use]
    pub fn new(
        message_duration: Option<Duration>,
        relay_leader_duration: Duration,
        signing_leader_duration: Duration,
    ) -> Self {
        let message_duration = message_duration.unwrap_or_else(|| {
            let duration = random::<u8>() % 10 + 5;
            Duration::from_secs(duration.into())
        });
        let key = identity::Keypair::generate_ed25519();
        let peer_id = key.public().into();

        Self {
            message_duration,
            relay_leader_duration,
            signing_leader_duration,
            key,
            peer_id,
            peers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// # Errors
    ///
    /// return error when fail to create swarm
    pub async fn start(
        self,
        mut shutdown_signal: ShutdownSignal,
        mut stdin_receiver: mpsc::Receiver<String>,
    ) -> Result<()> {
        let message_topic = Topic::new(MESSAGE_TOPIC);
        let mut message_timer = time::interval(self.message_duration);
        message_timer.reset();

        tracing::info!(
            "Start Peer:  peer_id={}, message_duration={}",
            self.peer_id,
            self.message_duration.as_secs()
        );

        let mut behaviour = PeerBehaviour {
            floodsub: Floodsub::new(self.peer_id.clone()),
            mdns: Mdns::new(
                MdnsConfig {
                    ttl: Duration::from_secs(60),
                    query_interval: Duration::from_secs(30),
                    enable_ipv6: false,
                },
                self.peer_id.clone(),
            )
            .expect("must create mdns"),
        };

        behaviour.floodsub.subscribe(message_topic.clone());

        let mut swarm = SwarmBuilder::with_existing_identity(self.key.clone())
            .with_tokio()
            .with_tcp(TcpConfig::default(), NoiseConfig::new, YamuxConfig::default)
            .context(error::SwarmWithTcpSnafu)?
            .with_behaviour(|_key| behaviour)
            .expect("swarm with behaviour is infallible")
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        // let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();

        Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse().expect("can get a local socket"))
            .expect("swarm can be started");

        loop {
            let action = {
                tokio::select! {
                    _ = message_timer.tick() => Action::MessageTrigger,
                    line = stdin_receiver.recv() => Action::Input(line),
                    event = swarm.select_next_some() => {
                        Action::Swarm(event)
                    },
                    () = shutdown_signal.wait() => Action::Stop,
                }
            };

            match action {
                Action::Input(line) => {
                    if let Some(line) = line {
                        match line.as_str() {
                            cmd if cmd.starts_with("ls p") => handle_list_peers(&mut swarm).await,
                            _ => tracing::error!("unknown command from stdin"),
                        }
                    } else {
                        tracing::warn!("stdin_sender is closed");
                    }
                }
                Action::MessageTrigger => {
                    tracing::warn!("Message trigger");
                    let message = format!("Message from {}", self.peer_id.clone());
                    swarm.behaviour_mut().floodsub.publish(message_topic.clone(), message);
                }
                Action::Swarm(swarm_event) => match swarm_event {
                    SwarmEvent::Behaviour(PeerBehaviourEvent::Floobsub(
                        FloodsubEvent::Message(msg),
                    )) => {
                        let message = String::from_utf8_lossy(&msg.data);
                        tracing::warn!("Receive message from {:?}: {message}", msg.source);
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
                    _ => {
                        tracing::info!("Unhandled event {swarm_event:?}")
                    }
                },
                Action::Stop => break,
            }
        }

        tracing::warn!("PeerWorker is stopped");

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
