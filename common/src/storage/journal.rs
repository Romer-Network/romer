use commonware_storage::journal::{self, Journal};
use commonware_runtime::tokio::{self, Blob, Context};
use prometheus_client::registry::Registry;
use serde::{Serialize, Deserialize};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::types::org::{Organization, OrganizationType};

#[derive(Serialize, Deserialize)]
enum JournalEntry {
    OrganizationRegistered(Organization),
    OrganizationUpdated(Organization),
    OrganizationDeactivated(String),
}

pub struct RomerJournal {
    journal: Journal<tokio::Blob, tokio::Context>,
}

impl RomerJournal {
    pub async fn new() -> Result<Self, String> {
        let runtime_cfg = tokio::Config {
            storage_directory: "devnet-storage".into(),
            ..Default::default()
        };
        
        let (executor, runtime) = tokio::Executor::init(runtime_cfg);
        
        let journal = Journal::init(
            runtime,  
            journal::Config {
                registry: Arc::new(Mutex::new(Registry::default())),
                partition: String::from("organizations"),
            },
        )
        .await
        .map_err(|e| e.to_string())?;

        Ok(Self { journal })
    }

    pub async fn write_organization(&mut self, org: Organization) -> Result<(), String> {
        let entry = JournalEntry::OrganizationRegistered(org);
        let bytes = serde_json::to_vec(&entry).map_err(|e| e.to_string())?;
        
        self.journal.append(1, bytes.into()).await.map_err(|e| e.to_string())?;
        self.journal.sync(1).await.map_err(|e| e.to_string())?;
        
        Ok(())
    }
}