use std::io::{self, Write};
use serde::de::value;
use uuid::Uuid;
use romer_common::{
    error::RomerResult, keystore::keymanager::KeyManager, storage::journal::RomerJournal, types::org::{Organization, OrganizationType}
};
use crate::handlers::Handler;

/// Handler for registering new SenderCompID entries. This handler modifies
/// system state by adding new organizations to the journal.
pub struct RegisterSenderCompIdHandler {
    journal: RomerJournal,
}

impl RegisterSenderCompIdHandler {
    pub async fn new() -> io::Result<Self> {
        let journal = RomerJournal::new()
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        Ok(Self { journal })
    }

    /// Prompts for and validates organization name
    fn get_org_name(&self) -> io::Result<String> {
        println!("\nEnter organization name:");
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let name = input.trim().to_string();
        if name.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput, 
                "Organization name cannot be empty"
            ));
        }
        
        Ok(name)
    }

    /// Displays organization type options and handles selection
    fn get_org_type(&self) -> io::Result<OrganizationType> {
        println!("\nSelect organization type:");
        println!("1. Market Maker");
        println!("2. Broker Dealer");
        println!("3. Bank");
        println!("4. Asset Manager");
        println!("5. Infrastructure Provider");
        println!("6. Service Provider");
        println!("7. Prime Broker");
        println!("8. Custodian");
        
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim() {
            "1" => Ok(OrganizationType::MarketMaker),
            "2" => Ok(OrganizationType::BrokerDealer),
            "3" => Ok(OrganizationType::Bank),
            "4" => Ok(OrganizationType::AssetManager),
            "5" => Ok(OrganizationType::InfraProvider),
            "6" => Ok(OrganizationType::ServiceProvider),
            "7" => Ok(OrganizationType::PrimeBroker),
            "8" => Ok(OrganizationType::Custodian),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput, 
                "Invalid organization type selected"
            )),
        }
    }

    /// Prompts for and validates SenderCompID
    fn get_sender_comp_id(&self) -> io::Result<String> {
        println!("\nEnter desired SenderCompID:");
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let sender_comp_id = input.trim().to_string();
        if sender_comp_id.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput, 
                "SenderCompID cannot be empty"
            ));
        }
        
        // Additional validation could be added here:
        // - Check for allowed characters
        // - Verify length limits
        // - Enforce formatting rules
        
        Ok(sender_comp_id)
    }

    /// Displays organization details and gets confirmation for registration
    fn confirm_registration(&self, org: &Organization) -> io::Result<bool> {
        println!("\nPlease confirm organization registration:");
        println!("Name: {}", org.name);
        println!("Type: {:?}", org.org_type);
        println!("SenderCompID: {}", org.sender_comp_id);
        println!("\nProceed with registration? (y/n)");
        
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        Ok(input.trim().to_lowercase() == "y")
    }
}

impl Handler for RegisterSenderCompIdHandler {
    fn handle(&mut self) -> RomerResult<()> {
        // Create runtime for async operations
        let runtime = tokio::runtime::Runtime::new()?;
        
        Ok(runtime.block_on(async {
            // Collect all organization details from user
            let name = self.get_org_name()?;
            let org_type = self.get_org_type()?;
            let sender_comp_id = self.get_sender_comp_id()?;
            
            let id = Uuid::new_v4().to_string();

            // Setup the BLS Key
            let key_manager = KeyManager::new().expect("Failed to create Key Manager");
            let public_key = key_manager.get_bls_public_key().expect("Couldn't get BLS Key. Try create one."); 
            
            // Create and validate the organization instance
            let org = Organization::new(
                id,
                name,
                org_type,
                sender_comp_id,
                public_key,
            );
            
            if let Err(e) = org.validate() {
                return Err(io::Error::new(io::ErrorKind::InvalidData, e));
            }
            
            // Get confirmation before proceeding
            if !self.confirm_registration(&org)? {
                println!("Registration cancelled.");
                return Ok(());
            }
            
            // Write the organization to the journal
            if let Err(e) = self.journal.write_organization(org).await {
                return Err(io::Error::new(io::ErrorKind::Other, e));
            }
            
            println!("\nOrganization successfully registered!");
            Ok(())
        })?)
    }
}

pub struct GetStorageOrgsHandler {
    journal: RomerJournal,
}

impl GetStorageOrgsHandler {

}
/*  

Issue here is that we expect all Handler::handle functions to return a RomerResult. This is way too generic. Instead we should
be returning a ClientHandlerResult. 

The Handler trait should also be made more specific. So it should be called ClientInputHandler

# RomerJournal Refactoring
* new method - get_all_organizations

impl Handler for GetStorageOrgsHandler {
    pub fn handle(&mut self) -> RomerResult<()> {
        
    }
}
*/

