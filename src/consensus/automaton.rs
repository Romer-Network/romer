use bytes::{BufMut, Bytes, BytesMut};
use commonware_consensus::{simplex::Context, Automaton};
use commonware_consensus::{Committer, Relay, Supervisor};
use commonware_cryptography::{Ed25519, PublicKey, Scheme};
use commonware_p2p::authenticated::{self, Config as P2PConfig, Network};
use commonware_p2p::{Recipients, Sender};
use commonware_runtime::deterministic::Context as RuntimeContext;
use commonware_runtime::{Clock, Spawner};
use commonware_storage::journal::{Config as JournalConfig, Journal};
use futures::channel::oneshot;
use governor::Quota;
use prometheus_client::registry::Registry;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tracing::{info, warn};

use crate::block::{Block, BlockHeader, Transaction, TransactionType, TransferType};
use crate::config::genesis::GenesisConfig;
use crate::config::storage::StorageConfig;
use crate::config::tokenomics::TokenomicsConfig;
use crate::consensus::supervisor::BlockchainSupervisor;
use crate::utils::utils::BlockHasher;

#[derive(Debug, Serialize, Deserialize)]
enum GenesisBlockError {
    SerializationError(String),
    OtherError(String),
}
/// Core blockchain automaton responsible for block creation, validation, and network interactions
#[derive(Clone)]
pub struct BlockchainAutomaton {
    runtime: RuntimeContext,
    p2p_sender: Option<commonware_p2p::authenticated::Sender>,
    pub signer: Ed25519,
    genesis_config: GenesisConfig,
    storage_config: StorageConfig,
    tokenomics_config: TokenomicsConfig,
    pub supervisor: BlockchainSupervisor,
}

impl BlockchainAutomaton {
    pub fn new(
        runtime: RuntimeContext,
        signer: Ed25519,
        genesis_config: GenesisConfig,
        storage_config: StorageConfig,
        tokenomics_config: TokenomicsConfig,
    ) -> Self {
        let supervisor_signer = signer.clone();

        Self {
            runtime,
            p2p_sender: None,
            signer,
            genesis_config,
            storage_config,
            tokenomics_config,
            supervisor: BlockchainSupervisor::new(supervisor_signer.public_key()),
        }
    }

    pub async fn run(
        &mut self,
        address: SocketAddr,
        bootstrap: Option<SocketAddr>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Log the start of node initialization
        info!("Starting Rømer Chain Node at {}", address);

        let runtime = self.runtime.clone();
        let signer = self.signer.clone();
        let genesis_config = self.genesis_config.clone();
        let storage_config = self.storage_config.clone();
        let tokenomics_config = self.tokenomics_config.clone();

        // Validate and initialize storage directories
        self.storage_config
            .initialize_directories()
            .map_err(|e| format!("Storage directory initialization failed: {}", e))?;

        // Create the Journal configuration for genesis
        let journal_cfg = JournalConfig {
            registry: Arc::new(Mutex::new(Registry::default())),
            partition: self.storage_config.journal.partitions.genesis.clone(),
        };

        // Initialize the Journal for genesis data
        let mut journal = Journal::init(self.runtime.clone(), journal_cfg)
            .await
            .map_err(|e| format!("Journal initialization failed: {}", e))?;

        // Create the genesis block
        let genesis_block = self
            .create_genesis_block(self.genesis_config.network.genesis_time)
            .await
            .map_err(|e| format!("Genesis block creation failed: {:?}", e))?;

        // Serialize and persist the genesis block
        let serialized_block = bincode::serialize(&genesis_block)
            .map_err(|e| format!("Genesis block serialization failed: {}", e))?;

        // Append genesis block to journal
        journal
            .append(0, Bytes::from(serialized_block))
            .await
            .map_err(|e| format!("Failed to append genesis block: {}", e))?;

        // Close the journal to ensure persistence
        journal
            .close()
            .await
            .map_err(|e| format!("Journal close failed: {}", e))?;

        // Initialize P2P network configuration
        let p2p_cfg = authenticated::Config::aggressive(
            self.signer.clone(),
            b"romer-network", // Unique namespace to prevent replay attacks
            Arc::new(Mutex::new(Registry::default())),
            address,
            bootstrap.map_or(vec![], |addr| vec![(self.signer.public_key(), addr)]),
            self.genesis_config.networking.max_message_size,
        );

        // Start the network
        let (mut network, mut oracle) = Network::new(self.runtime.clone(), p2p_cfg);

        // Register the initial validator set
        oracle.register(0, vec![self.signer.public_key()]);

        // Register network channels
        let (sender, receiver) = network.register(
            0,
            Quota::per_second(NonZeroU32::new(1).unwrap()),
            self.genesis_config.networking.max_message_backlog,
            Some(self.genesis_config.networking.compression_level),
        );

        // Set the P2P sender in the automaton
        self.set_sender(sender);

        // Spawn network handler
        let network_handler = self.runtime.spawn("network", network.run());

        // Prepare for consensus
        let consensus_handler = self.runtime.spawn("consensus", async move {
            // Create a new instance for consensus to avoid borrowing issues
            let mut consensus_automaton = BlockchainAutomaton::new(
                runtime.clone(),
                signer.clone(),
                genesis_config.clone(),
                storage_config.clone(),
                tokenomics_config.clone(),
            );

            // Start the Simplex consensus engine
            consensus_automaton
                .run_consensus(address, bootstrap)
                .await
                .expect("Consensus engine failed");
        });

        // Wait for network and consensus to complete
        tokio::select! {
            _ = network_handler => {},
            _ = consensus_handler => {},
        }

        Ok(())
    }

    // A method specifically for consensus initialization
    async fn run_consensus(
        &mut self,
        address: SocketAddr,
        bootstrap: Option<SocketAddr>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Specific consensus initialization logic
        info!("Starting consensus with address: {}", address);
        if let Some(bootstrap_addr) = bootstrap {
            info!("Using bootstrap address: {}", bootstrap_addr);
        }

        // Your existing consensus engine startup logic
        // This might involve calling .run() on your consensus implementation
        Ok(())
    }

    /// Set the P2P sender for network communication
    pub fn set_sender(&mut self, sender: commonware_p2p::authenticated::Sender) {
        self.p2p_sender = Some(sender);
    }

    async fn create_genesis_block(
        &mut self,
        genesis_time: u64,
    ) -> Result<Block, GenesisBlockError> {
        let mut block_hasher = BlockHasher::new();

        // Convert treasury address to Vec<u8> first
        let treasury_vec =
            block_hasher.address_to_bytes(&self.tokenomics_config.addresses.treasury);

        // Convert the Vec<u8> to fixed-size array for the transaction
        let mut treasury_bytes = [0u8; 32];
        // If the vector is shorter than 32 bytes, this will pad with zeros
        // If longer, it will take the first 32 bytes
        treasury_bytes[..treasury_vec.len().min(32)]
            .copy_from_slice(&treasury_vec[..treasury_vec.len().min(32)]);

        let mint_transaction = Transaction {
            transaction_type: TransactionType::TokenTransfer {
                to: treasury_bytes, // Now using fixed-size array
                amount: self.tokenomics_config.supply.initial_supply,
                transfer_type: TransferType::Mint,
            },
            from: [0u8; 32],
            nonce: 0,
            gas_amount: 0,
            signature: [0u8; 32],
        };

        let transactions_root =
            block_hasher.calculate_transactions_root(&[mint_transaction.clone()]);

        // For state root calculation, we can use the original Vec<u8>
        let initial_state = vec![(
            treasury_vec, // Using the vector directly here
            self.tokenomics_config.supply.initial_supply,
        )];

        let state_root = block_hasher.calculate_state_root(&initial_state);

        let public_key_bytes = self.signer.public_key();
        let mut validator_key = [0u8; 32];
        validator_key.copy_from_slice(&public_key_bytes);

        let block = Block {
            header: BlockHeader {
                view: 0,
                height: 0,
                timestamp: genesis_time,
                previous_hash: [0u8; 32],
                transactions_root,
                state_root,
                validator_public_key: validator_key,
            },
            transactions: vec![mint_transaction],
        };

        // Print detailed genesis block information
        info!("\n=== Genesis Block Created ===");
        info!("Block Header:");
        info!("  View: {}", block.header.view);
        info!("  Height: {}", block.header.height);
        info!("  Timestamp: {}", block.header.timestamp);
        info!(
            "  Previous Hash: 0x{}",
            hex::encode(block.header.previous_hash)
        );
        info!(
            "  Transactions Root: 0x{}",
            hex::encode(block.header.transactions_root)
        );
        info!("  State Root: 0x{}", hex::encode(block.header.state_root));
        info!(
            "  Validator Public Key: 0x{}",
            hex::encode(block.header.validator_public_key)
        );

        info!("\nTransactions:");
        for (i, tx) in block.transactions.iter().enumerate() {
            info!("Transaction {}:", i + 1);
            match &tx.transaction_type {
                TransactionType::TokenTransfer {
                    to,
                    amount,
                    transfer_type,
                } => {
                    info!("  Type: Token Transfer");
                    info!("  To: 0x{}", hex::encode(to));
                    info!("  Amount: {} Ole", amount); // Ole is the smallest unit of RØMER
                    info!("  Transfer Type: {:?}", transfer_type);
                }
            }
            info!("  From: 0x{}", hex::encode(tx.from));
            info!("  Nonce: {}", tx.nonce);
            info!("  Gas Amount: {}", tx.gas_amount);
            info!("  Signature: 0x{}", hex::encode(tx.signature));
        }

        info!("\nInitial State:");
        info!(
            "  Treasury Balance: {} Ole",
            self.tokenomics_config.supply.initial_supply
        );
        info!("  Treasury Address: 0x{}", hex::encode(treasury_bytes));

        info!("=== End Genesis Block ===\n");

        Ok(block)
    }

    async fn persist_genesis_block(
        &mut self,
        block: &Block,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Serialization logic
        let serialized_block = bincode::serialize(block).map_err(|e| Box::new(e))?;

        // Journal configuration
        let journal_cfg = JournalConfig {
            registry: Arc::new(Mutex::new(Registry::default())),
            partition: self.storage_config.journal.partitions.genesis.clone(),
        };

        // Initialize Journal
        let mut journal = Journal::init(self.runtime.clone(), journal_cfg).await?;

        // Append to journal
        journal.append(0, Bytes::from(serialized_block)).await?;

        // Close journal
        journal.close().await?;

        Ok(())
    }
}

// The rest of the trait implementations remain the same as in the previous version
impl Automaton for BlockchainAutomaton {
    type Context = Context;

    async fn genesis(&mut self) -> Bytes {
        // Use .await and .expect() or proper error handling
        let genesis_block = self
            .create_genesis_block(self.genesis_config.network.genesis_time)
            .await
            .expect("Failed to create genesis block"); // This will panic if block creation fails

        let mut buffer = BytesMut::new();
        buffer.put_u32(genesis_block.header.view);
        buffer.put_u64(genesis_block.header.height);
        buffer.put_u64(genesis_block.header.timestamp);
        buffer.put_slice(&genesis_block.header.previous_hash);
        buffer.put_slice(&genesis_block.header.transactions_root);
        buffer.put_slice(&genesis_block.header.state_root);
        buffer.put_slice(&genesis_block.header.validator_public_key);

        buffer.freeze()
    }
    // Changed to return the Future directly instead of nesting it
    async fn propose(&mut self, context: Self::Context) -> oneshot::Receiver<Bytes> {
        let timestamp: u64 = self
            .runtime
            .current()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let block = Bytes::from(format!("Block at view {}: {}", context.view, timestamp));

        if let Some(sender) = &mut self.p2p_sender {
            if let Err(e) = sender.send(Recipients::All, block.clone(), true).await {
                warn!("Failed to broadcast block: {}", e);
            }
        }

        // Create and return the receiver directly
        let (tx, rx) = oneshot::channel();
        let _ = tx.send(block);
        rx
    }

    // Changed to return the Future directly instead of nesting it
    async fn verify(&mut self, context: Self::Context, payload: Bytes) -> oneshot::Receiver<bool> {
        let is_valid = if payload.is_empty() {
            warn!("Received empty payload at view {}", context.view);
            false
        } else {
            match String::from_utf8(payload.to_vec()) {
                Ok(block_content) => {
                    let is_valid = block_content.contains(&format!("view {}", context.view));
                    if is_valid {
                        if let Some(sender) = &mut self.p2p_sender {
                            let validation_message = Bytes::from(format!(
                                "Block validated for view {}: {}",
                                context.view, block_content
                            ));
                            if let Err(e) =
                                sender.send(Recipients::All, validation_message, true).await
                            {
                                warn!("Failed to broadcast validation: {}", e);
                            }
                        }
                    }
                    is_valid
                }
                Err(_) => {
                    warn!("Invalid UTF-8 payload at view {}", context.view);
                    false
                }
            }
        };

        // Create and return the receiver directly
        let (tx, rx) = oneshot::channel();
        let _ = tx.send(is_valid);
        rx
    }
}

impl Relay for BlockchainAutomaton {
    async fn broadcast(&mut self, payload: Bytes) {
        if let Some(sender) = &mut self.p2p_sender {
            let mut sender = sender.clone();
            if let Err(e) = sender.send(Recipients::All, payload, true).await {
                warn!("Failed to broadcast: {}", e);
            }
        }
    }
}

impl Committer for BlockchainAutomaton {
    async fn prepared(&mut self, _proof: Bytes, payload: Bytes) {
        info!("Block prepared: {:?}", String::from_utf8_lossy(&payload));
    }

    async fn finalized(&mut self, _proof: Bytes, payload: Bytes) {
        info!("Block finalized: {:?}", String::from_utf8_lossy(&payload));
    }
}

impl Supervisor for BlockchainAutomaton {
    type Index = u64;
    type Seed = ();

    fn leader(&self, _index: Self::Index, _seed: Self::Seed) -> Option<PublicKey> {
        Some(self.signer.public_key())
    }

    fn participants(&self, _index: Self::Index) -> Option<&Vec<PublicKey>> {
        None
    }

    fn is_participant(&self, _index: Self::Index, _candidate: &PublicKey) -> Option<u32> {
        Some(0)
    }

    async fn report(&self, _activity: u8, _proof: Bytes) {}
}
