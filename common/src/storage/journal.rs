use commonware_runtime::tokio::{self, Blob, Context};
use commonware_storage::journal::{self, Journal};
use prometheus_client::registry::Registry;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::types::org::{Organization, OrganizationType};

#[derive(Serialize, Deserialize)]
pub enum JournalEntry {
    OrganizationRegistered(Organization),
    OrganizationUpdated(Organization),
    OrganizationDeactivated(String),
}

pub enum Partition {
    SYSTEM,
    TRADING,
}

pub enum Section {
    ORGANIZATION
}
pub struct RomerJournal {
    /// The core journal instance for storage and retrieval
    pub journal: Journal<tokio::Blob, tokio::Context>,
    
    /// The partition identifier for this journal
    pub partition: Partition,

    /// The section or subsystem within the partition
    pub section: Section,
}

impl RomerJournal {
    pub async fn new(
        partition: Partition,
        section: Section
    ) -> Result<Self, String> {
        let runtime_cfg = tokio::Config {
            storage_directory: "devnet-storage".into(),
            ..Default::default()
        };

        let (executor, runtime) = tokio::Executor::init(runtime_cfg);

        let journal = Journal::init(
            runtime,
            journal::Config {
                registry: Arc::new(Mutex::new(Registry::default())),
                partition: String::from("system"),
            },
        )
        .await
        .map_err(|e| e.to_string())?;

        Ok(Self { 
            journal,
            partition,
            section,
         })
    }

}
