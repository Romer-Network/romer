/* mod application;
mod gui;
mod node;
mod validation;
mod types;


use commonware_consensus::simplex::{self, Engine, Prover};
use commonware_cryptography::{Ed25519, Scheme, Sha256};
use commonware_p2p::authenticated::{self, Network};
use commonware_runtime::{
    tokio::{self, Executor},
    Runner, Spawner,
};
use commonware_storage::journal::{self, Journal};
use commonware_utils::{hex, union};
use governor::Quota;
use node::cmd::cli;
use prometheus_client::registry::Registry;
use common::utils::hardware_validator::HardwareDetector;
use std::sync::{Arc, Mutex};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    num::NonZeroU32,
};
use std::{str::FromStr, time::Duration};

/// Unique namespace to avoid message replay attacks.
const APPLICATION_NAMESPACE: &[u8] = b"ROMER";

fn main() {
    let app_config = cli::setup_clap_command();

    // Create GUI
    let gui = gui::Gui::new();
    
    // TODO: Replace this with getting the Signer from NodeKeyManager
    let signer = Ed25519::from_seed(app_config.me.0.parse::<u64>().expect("Invalid node ID"));
    tracing::info!(key = hex(&signer.public_key()), "loaded signer");

    // Configure my port
    let port = app_config.me.1.port();
    tracing::info!(port, "loaded port");

    // Configure allowed peers
    let mut validators = Vec::new();
    for peer in app_config.participants {
        let verifier = Ed25519::from_seed(peer).public_key();
        tracing::info!(key = hex(&verifier), "registered authorized key",);
        validators.push(verifier);
    }

    // Configure bootstrappers (if provided)
    let mut bootstrapper_identities = Vec::new();
    for bootstrapper in app_config.bootstrappers {
        let parts = bootstrapper.split('@').collect::<Vec<&str>>();
        let bootstrapper_key = parts[0]
            .parse::<u64>()
            .expect("Bootstrapper key not well-formed");
        let verifier = Ed25519::from_seed(bootstrapper_key).public_key();
        let bootstrapper_address =
            SocketAddr::from_str(parts[1]).expect("Bootstrapper address not well-formed");
        bootstrapper_identities.push((verifier, bootstrapper_address));
    }

    // Configure storage directory
    let storage_directory = app_config.storage_dir;

    // Initialize runtime
    let runtime_cfg = tokio::Config {
        storage_directory: storage_directory.into(),
        ..Default::default()
    };
    let (executor, runtime) = Executor::init(runtime_cfg.clone());

    // Configure network
    let p2p_cfg = authenticated::Config::aggressive(
        signer.clone(),
        &union(APPLICATION_NAMESPACE, b"_P2P"),
        Arc::new(Mutex::new(Registry::default())),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
        bootstrapper_identities.clone(),
        1024 * 1024, // 1MB
    );

    // Start runtime
    executor.start(async move {
        let (mut network, mut oracle) = Network::new(runtime.clone(), p2p_cfg);

        // Provide authorized peers
        //
        // In a real-world scenario, this would be updated as new peer sets are created (like when
        // the composition of a validator set changes).
        oracle.register(0, validators.clone()).await;

        // Register consensus channels
        //
        // If you want to maximize the number of views per second, increase the rate limit
        // for this channel.
        let (voter_sender, voter_receiver) = network.register(
            0,
            Quota::per_second(NonZeroU32::new(10).unwrap()),
            256, // 256 messages in flight
            Some(3),
        );
        let (resolver_sender, resolver_receiver) = network.register(
            1,
            Quota::per_second(NonZeroU32::new(10).unwrap()),
            256, // 256 messages in flight
            Some(3),
        );

        // Initialize storage
        let journal = Journal::init(
            runtime.clone(),
            journal::Config {
                registry: Arc::new(Mutex::new(Registry::default())),
                partition: String::from("log"),
            },
        )
        .await
        .expect("Failed to initialize journal");

        // Initialize application
        let namespace = union(APPLICATION_NAMESPACE, b"_CONSENSUS");
        let hasher = Sha256::default();
        let prover: Prover<Ed25519, Sha256> = Prover::new(&namespace);
        let (application, supervisor, mailbox) = application::Application::new(
            runtime.clone(),
            application::Config {
                prover,
                hasher: hasher.clone(),
                mailbox_size: 1024,
                participants: validators.clone(),
                validator_location: Some(app_config.location),
            },
        );

        // Initialize consensus
        let engine = Engine::new(
            runtime.clone(),
            journal,
            simplex::Config {
                crypto: signer.clone(),
                hasher,
                automaton: mailbox.clone(),
                relay: mailbox.clone(),
                committer: mailbox,
                supervisor,
                registry: Arc::new(Mutex::new(Registry::default())),
                namespace,
                mailbox_size: 1024,
                replay_concurrency: 1,
                leader_timeout: Duration::from_secs(1),
                notarization_timeout: Duration::from_secs(2),
                nullify_retry: Duration::from_secs(10),
                fetch_timeout: Duration::from_secs(1),
                activity_timeout: 10,
                max_fetch_count: 32,
                max_fetch_size: 1024 * 512,
                fetch_concurrent: 2,
                fetch_rate_per_peer: Quota::per_second(NonZeroU32::new(1).unwrap()),
            },
        );

        // Start consensus
        runtime.spawn("application", application.run());
        runtime.spawn("network", network.run());
        runtime.spawn(
            "engine",
            engine.run(
                (voter_sender, voter_receiver),
                (resolver_sender, resolver_receiver),
            ),
        );

        // Block on GUI
        gui.run(runtime).await;
    });
}

*/

fn main() {
    println!("Hello World");
}