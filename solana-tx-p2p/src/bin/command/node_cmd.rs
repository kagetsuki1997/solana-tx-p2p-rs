use std::{process, sync::Arc, time::Duration};

use clap::Args;
use futures_util::TryFutureExt;
use snafu::ResultExt;
use solana_sdk::pubkey::Pubkey;
use solana_tx_p2p::{
    service::{
        create_solana_client, start_heartbeat_trigger, start_message_trigger, PeerWorker,
        PeerWorkerInboundEvent, RRElectionWorker, RRElectionWorkerType, SolanaRelayer,
        SolanaSigner,
    },
    ShutdownSignal, SignalHandleBuilder,
};
use tokio::{
    runtime::Runtime,
    sync::{mpsc, RwLock},
    task::JoinSet,
};

use crate::{
    env,
    error::{self, Error, Result},
    tracing::init_tracing,
};

const APP_NAME: &str = "Solana Transaction Peer-to-peer Node";

#[derive(Args, Debug)]
pub struct NodeCmd {
    #[arg(
        name = "message-duration",
        long,
        env = env::MESSAGE_DURATION,
    )]
    message_duration: Option<humantime::Duration>,

    #[arg(
        name = "relay-leader-duration",
        long,
        env = env::RELAY_LEADER_DURATION,
        default_value = "60s"
    )]
    relay_leader_duration: humantime::Duration,

    #[arg(
        name = "signing-leader-duration",
        long,
        env = env::SIGNING_LEADER_DURATION,
        default_value = "60s"
    )]
    signing_leader_duration: humantime::Duration,

    #[arg(
        name = "heartbeat-duration",
        long,
        env = env::HEARTBEAT_DURATION,
        default_value = "1s"
    )]
    heartbeat_duration: humantime::Duration,

    #[command(flatten)]
    solana: Solana,
}

#[derive(Args, Debug)]
pub struct Solana {
    #[arg(
        name = "solana-program-id",
        long,
        env = env::SOLANA_PROGRAM_ID
    )]
    program_id: Pubkey,

    #[arg(
        name = "solana-rpc-url",
        long,
        env = env::SOLANA_RPC_URL
    )]
    rpc_url: String,
}

impl NodeCmd {
    /// Run the node
    // FIXME: clippy::significant_drop_tightening: clippy bug
    #[allow(clippy::significant_drop_tightening)]
    pub fn run(self) -> Result<()> {
        Runtime::new().context(error::InitializeAsyncRuntimeSnafu)?.block_on(async {
            let _handle = init_tracing("debug,hyper=info,tower=info")?;

            tracing::debug!("{APP_NAME} starting");
            tracing::info!("Process ID: {}", process::id());

            let mut join_set = JoinSet::<solana_tx_p2p::Result<()>>::new();

            tracing::info!("Initializing shutdown signal handler");
            let shutdown_signal_handler = SignalHandleBuilder::new(None).start();
            let shutdown_signal = shutdown_signal_handler.shutdown_signal();

            let (stdin_sender, stdin_receiver) = mpsc::channel(10);

            tracing::info!("Initializing P2P Node");
            let _peer_worker_inbound_sender =
                self.start_node(&mut join_set, shutdown_signal, stdin_receiver).await?;

            // stdin reader
            let rt = Runtime::new().context(error::InitializeAsyncRuntimeSnafu)?;
            rt.spawn_blocking(move || {
                loop {
                    let mut buffer = String::new();
                    if let Err(err) = std::io::stdin().read_line(&mut buffer) {
                        tracing::error!("Fail to read stdin: {err}");
                        continue;
                    }

                    if let Err(err) = stdin_sender.blocking_send(buffer) {
                        tracing::error!("Fail to send stdin: {err}");
                        break;
                    }
                }

                tracing::warn!("stdin reader thread stopped");
            });

            while let Some(result) = join_set.join_next().await {
                match result {
                    Ok(Ok(())) => {}
                    Ok(Err(err)) => tracing::error!("Server error: {err}"),
                    Err(err) => tracing::error!("Join error: {err}"),
                }
            }

            shutdown_signal_handler.stop();
            rt.shutdown_background();

            Result::<_, Error>::Ok(())
        })?;

        tracing::info!("{APP_NAME} shutdown complete");

        Ok(())
    }

    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    pub async fn start_node(
        self,
        join_set: &mut JoinSet<solana_tx_p2p::Result<()>>,
        shutdown_signal: ShutdownSignal,
        stdin_receiver: mpsc::Receiver<String>,
    ) -> Result<mpsc::Sender<PeerWorkerInboundEvent>> {
        let Self {
            message_duration,
            relay_leader_duration,
            signing_leader_duration,
            heartbeat_duration,
            solana,
        } = self;

        let peers = Arc::new(RwLock::new(Vec::new()));
        let signer = Arc::new(RwLock::new("signer".to_string()));
        let relayer = Arc::new(RwLock::new("relayer".to_string()));

        let (relayer_heartbeat_sender, relayer_heartbeat_receiver) = mpsc::channel(100);
        let (relayer_election_worker_inbound_sender, relayer_election_worker_inbound_receiver) =
            mpsc::channel(100);

        let (signer_heartbeat_sender, signer_heartbeat_receiver) = mpsc::channel(100);
        let (signer_election_worker_inbound_sender, signer_election_worker_inbound_receiver) =
            mpsc::channel(100);

        let (solana_signer_inbound_sender, solana_signer_inbound_receiver) = mpsc::channel(100);
        let (solana_relayer_inbound_sender, solana_relayer_inbound_receiver) = mpsc::channel(100);
        let (keypair, solana_keypair) = PeerWorker::generate_keypair();

        tracing::info!("Initializing Solana client");
        let solana_client = create_solana_client(&solana.rpc_url, solana_keypair.clone())
            .await
            .context(error::CreateSolanaClientSnafu)?;

        tracing::info!("Initializing PeerWorker");
        let peer_worker = PeerWorker::new(
            keypair,
            peers.clone(),
            relayer.clone(),
            signer.clone(),
            relayer_heartbeat_sender,
            signer_heartbeat_sender,
            relayer_election_worker_inbound_sender,
            signer_election_worker_inbound_sender,
            solana_relayer_inbound_sender,
            solana_signer_inbound_sender,
            solana_client.clone(),
        );
        let peer_worker_inbound_sender = peer_worker.peer_worker_inbound_sender();
        let peer_id = peer_worker.peer_id();

        join_set
            .build_task()
            .name("peer worker")
            .spawn(peer_worker.start(shutdown_signal.clone(), stdin_receiver).err_into())
            .context(error::SpawnSnafu { name: "peer worker".to_string() })?;

        tracing::info!("Initializing relay leader election worker");
        let relayer_election_worker = RRElectionWorker::new(
            RRElectionWorkerType::Relayer,
            relayer.clone(),
            heartbeat_duration.saturating_add(Duration::from_secs(3)),
            *relay_leader_duration,
            relayer_election_worker_inbound_receiver,
            relayer_heartbeat_receiver,
            peers.clone(),
            peer_worker_inbound_sender.clone(),
        );
        join_set
            .build_task()
            .name("relay leader election worker")
            .spawn(relayer_election_worker.start(shutdown_signal.clone()).err_into())
            .context(error::SpawnSnafu { name: "relay leader election worker".to_string() })?;

        tracing::info!("Initializing signing leader election worker");
        let signer_election_worker = RRElectionWorker::new(
            RRElectionWorkerType::Signer,
            signer.clone(),
            heartbeat_duration.saturating_add(Duration::from_secs(3)),
            *signing_leader_duration,
            signer_election_worker_inbound_receiver,
            signer_heartbeat_receiver,
            peers,
            peer_worker_inbound_sender.clone(),
        );
        join_set
            .build_task()
            .name("signing leader election worker")
            .spawn(signer_election_worker.start(shutdown_signal.clone()).err_into())
            .context(error::SpawnSnafu { name: "signing leader election worker".to_string() })?;

        tracing::info!("Initializing SolanaSigner");
        let solana_signer = SolanaSigner::new(
            peer_id,
            signer,
            solana_keypair.clone(),
            peer_worker_inbound_sender.clone(),
            solana.program_id,
            solana_client.clone(),
            solana_signer_inbound_receiver,
        );
        join_set
            .build_task()
            .name("solana signer")
            .spawn(solana_signer.start(shutdown_signal.clone()).err_into())
            .context(error::SpawnSnafu { name: "solana signer".to_string() })?;

        tracing::info!("Initializing SolanaRelayer");
        let solana_relayer = SolanaRelayer::new(
            peer_id,
            relayer,
            peer_worker_inbound_sender.clone(),
            solana_client,
            solana_relayer_inbound_receiver,
        );
        join_set
            .build_task()
            .name("solana relayer")
            .spawn(solana_relayer.start(shutdown_signal.clone()).err_into())
            .context(error::SpawnSnafu { name: "solana relayer".to_string() })?;

        tracing::info!("Initializing message trigger task");
        join_set
            .build_task()
            .name("message trigger")
            .spawn(
                start_message_trigger(
                    message_duration.as_deref().copied(),
                    shutdown_signal.clone(),
                    peer_worker_inbound_sender.clone(),
                )
                .err_into(),
            )
            .context(error::SpawnSnafu { name: "message trigger".to_string() })?;

        tracing::info!("Initializing heartbeat trigger task");
        join_set
            .build_task()
            .name("heartbeat trigger")
            .spawn(
                start_heartbeat_trigger(
                    *self.heartbeat_duration,
                    shutdown_signal,
                    peer_worker_inbound_sender.clone(),
                )
                .err_into(),
            )
            .context(error::SpawnSnafu { name: "heartbeat trigger".to_string() })?;

        Ok(peer_worker_inbound_sender)
    }
}
