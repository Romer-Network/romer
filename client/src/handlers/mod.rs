use commonware_cryptography::{Bls12381, Ed25519, PrivateKey, Scheme};
use commonware_utils::hex;
use romer_common::keystore::keymanager::KeyManager;
use romer_common::types::keymanager::{SessionKeyData, SignatureScheme};
use std::fs;
use std::io::{self, Write};

pub trait Handler {
    fn handle(&self) -> io::Result<()>;
}

pub struct GenerateKeypairHandler {
    key_manager: KeyManager,
}

impl GenerateKeypairHandler {
    pub fn new() -> Result<Self, io::Error> {
        let key_manager =
            KeyManager::new().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        Ok(Self { key_manager })
    }

    fn get_key_type(&self) -> io::Result<SignatureScheme> {
        println!("\nSelect key type:");
        println!("1. Ed25519");
        println!("2. BLS12381");
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim() {
            "1" => Ok(SignatureScheme::Ed25519),
            "2" => Ok(SignatureScheme::Bls12381),
            _ => {
                println!("Invalid selection, defaulting to Ed25519");
                Ok(SignatureScheme::Ed25519)
            }
        }
    }
}

impl Handler for GenerateKeypairHandler {
    fn handle(&self) -> io::Result<()> {
        let scheme = self.get_key_type()?;

        match self.key_manager.initialize(scheme) {
            Ok(public_key) => {
                println!("Key generated successfully!");
                println!("Public key: {}", hex(&public_key));
                Ok(())
            }
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
        }
    }
}

pub struct CheckKeysHandler {
    key_manager: KeyManager,
}

impl CheckKeysHandler {
    pub fn new() -> Result<Self, io::Error> {
        let key_manager =
            KeyManager::new().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        Ok(Self { key_manager })
    }

    fn check_permanent_keys(&self) -> io::Result<()> {
        println!("\nChecking permanent keys...");

        // Try loading Ed25519 key
        match self
            .key_manager
            .load_permanent_key(SignatureScheme::Ed25519)
        {
            Ok(_) => println!("✓ Ed25519 key found"),
            Err(_) => println!("✗ No Ed25519 key found"),
        }

        // Try loading BLS key
        match self
            .key_manager
            .load_permanent_key(SignatureScheme::Bls12381)
        {
            Ok(_) => println!("✓ BLS12381 key found"),
            Err(_) => println!("✗ No BLS12381 key found"),
        }

        Ok(())
    }

    fn check_session_keys(&self) -> io::Result<()> {
        println!("\nChecking session keys...");

        // Get the sessions directory from KeyManager
        let sessions_dir = self.key_manager.session_dir.clone();

        // Read all files in the sessions directory
        let entries = match fs::read_dir(&sessions_dir) {
            Ok(entries) => entries,
            Err(_) => {
                println!("No session keys found");
                return Ok(());
            }
        };

        let mut found_sessions = false;

        // Process each session key file
        for entry in entries {
            found_sessions = true;
            let entry = entry?;
            let file_name = entry.file_name();
            let session_id = file_name.to_string_lossy().replace(".json", "");

            match self.key_manager.load_session_key(&session_id) {
                Ok(session_data) => {
                    println!("\nSession Key:");
                    println!("  ID: {}", session_id);
                    println!("  Purpose: {}", session_data.purpose);
                    println!("  Created: {}", session_data.created_at);
                    println!("  Expires: {}", session_data.expires_at);
                    println!("  Namespace: {}", session_data.namespace);
                }
                Err(e) => println!("Error loading session key {}: {}", session_id, e),
            }
        }

        if !found_sessions {
            println!("No session keys found");
        }

        Ok(())
    }
}

impl Handler for CheckKeysHandler {
    fn handle(&self) -> io::Result<()> {
        // Show base directory information
        println!("\nKey Storage Locations:");
        println!("Base Directory: {}", self.key_manager.base_dir.display());
        println!(
            "Permanent Keys: {}",
            self.key_manager.permanent_dir.display()
        );
        println!("Session Keys: {}", self.key_manager.session_dir.display());

        // Check permanent keys
        self.check_permanent_keys()?;

        // Check session keys
        self.check_session_keys()?;

        Ok(())
    }
}

pub struct SignMessageHandler {
    key_manager: KeyManager,
}

impl SignMessageHandler {
    pub fn new() -> Result<Self, io::Error> {
        let key_manager =
            KeyManager::new().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        Ok(Self { key_manager })
    }

    // Helper to get the key type selection from the user
    fn get_key_type(&self) -> io::Result<SignatureScheme> {
        println!("\nSelect key type to sign with:");
        println!("1. Ed25519");
        println!("2. BLS12381");
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim() {
            "1" => Ok(SignatureScheme::Ed25519),
            "2" => Ok(SignatureScheme::Bls12381),
            _ => {
                println!("Invalid selection, please try again");
                self.get_key_type()
            }
        }
    }

    // Helper to get the message to sign from the user
    fn get_message(&self) -> io::Result<String> {
        println!("\nEnter the message to sign:");
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    // Helper to list available keys and let user select one
    fn select_key(&self, scheme: SignatureScheme) -> io::Result<Vec<u8>> {
        // First check if we have any keys of this type
        let key_bytes = match self.key_manager.load_permanent_key(scheme) {
            Ok(bytes) => bytes,
            Err(_) => {
                println!("No {:?} keys found. Please generate one first.", scheme);
                return Err(io::Error::new(io::ErrorKind::NotFound, "No keys available"));
            }
        };

        // For now we just return the key bytes since we only support one key per type
        // In the future, we could list multiple keys and let the user select one
        Ok(key_bytes)
    }

    // Helper to sign the message with the selected key
    fn sign_message(
        &self,
        scheme: SignatureScheme,
        key_bytes: Vec<u8>,
        message: &str,
    ) -> io::Result<Vec<u8>> {
        match scheme {
            SignatureScheme::Ed25519 => {
                // Create an Ed25519 signer from the private key bytes
                let private_key = PrivateKey::from(key_bytes);
                let mut signer = <Ed25519 as Scheme>::from(private_key).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid Ed25519 key")
                })?;

                // Sign the message
                Ok(signer.sign(&[], message.as_bytes()).to_vec())
            }
            SignatureScheme::Bls12381 => {
                // Create a BLS signer from the private key bytes
                let private_key = PrivateKey::from(key_bytes);
                let mut signer = <Bls12381 as Scheme>::from(private_key)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid BLS key"))?;

                // Sign the message
                Ok(signer.sign(&[], message.as_bytes()).to_vec())
            }
        }
    }
}

impl Handler for SignMessageHandler {
    fn handle(&self) -> io::Result<()> {
        // Get the key type from the user
        let scheme = self.get_key_type()?;

        // Let the user select a key
        let key_bytes = self.select_key(scheme)?;

        // Get the message to sign
        let message = self.get_message()?;

        // Sign the message
        match self.sign_message(scheme, key_bytes, &message) {
            Ok(signature) => {
                println!("\nMessage signed successfully!");
                println!("Signature (hex): {}", hex(&signature));
                Ok(())
            }
            Err(e) => {
                println!("Error signing message: {}", e);
                Err(e)
            }
        }
    }
}

pub struct CreateSessionKeyHandler {
    key_manager: KeyManager,
}

impl CreateSessionKeyHandler {
    pub fn new() -> Result<Self, io::Error> {
        let key_manager =
            KeyManager::new().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        Ok(Self { key_manager })
    }

    // Helper to load and verify the parent BLS key exists
    fn load_parent_key(&self) -> io::Result<Vec<u8>> {
        match self
            .key_manager
            .load_permanent_key(SignatureScheme::Bls12381)
        {
            Ok(key_bytes) => Ok(key_bytes),
            Err(_) => {
                println!("No BLS key found. Please generate one first using the Generate Keypair option.");
                Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "BLS parent key not found",
                ))
            }
        }
    }

    // Helper to get the namespace from the user
    fn get_namespace(&self) -> io::Result<String> {
        println!("\nEnter the namespace for the session key (e.g., 'trading', 'settlement'):");
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let namespace = input.trim().to_string();

        // Basic validation - ensure namespace isn't empty and contains valid characters
        if namespace.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Namespace cannot be empty",
            ));
        }

        if !namespace
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Namespace can only contain alphanumeric characters, underscores, and hyphens",
            ));
        }

        Ok(namespace)
    }

    // Helper to get the duration in hours from the user
    fn get_duration(&self) -> io::Result<i64> {
        println!("\nEnter the session duration in hours (1-720):");
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().parse::<i64>() {
            Ok(hours) if hours >= 1 && hours <= 720 => Ok(hours),
            Ok(_) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Duration must be between 1 and 720 hours",
            )),
            Err(_) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid number format",
            )),
        }
    }

    // Helper to get the purpose description from the user
    fn get_purpose(&self) -> io::Result<String> {
        println!("\nEnter the purpose for this session key (e.g., 'Market making on DEX'):");
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let purpose = input.trim().to_string();

        if purpose.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Purpose cannot be empty",
            ));
        }

        Ok(purpose)
    }

    // Helper to confirm session key creation with the user
    fn confirm_creation(&self, namespace: &str, duration: i64, purpose: &str) -> io::Result<bool> {
        println!("\nPlease confirm session key creation:");
        println!("  Namespace: {}", namespace);
        println!("  Duration: {} hours", duration);
        println!("  Purpose: {}", purpose);
        println!("\nCreate session key with these parameters? (y/n)");
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        Ok(input.trim().to_lowercase() == "y")
    }

    // Helper to display the created session key information
    fn display_session_key(&self, session_data: &SessionKeyData) {
        println!("\nSession key created successfully!");
        println!("Key Information:");
        println!("  Created: {}", session_data.created_at);
        println!("  Expires: {}", session_data.expires_at);
        println!("  Public Key: {}", hex(&session_data.key_bytes));
        println!(
            "  Parent Public Key: {}",
            hex(&session_data.parent_public_key)
        );
        println!("  Namespace: {}", session_data.namespace);
        println!("  Purpose: {}", session_data.purpose);
    }
}

impl Handler for CreateSessionKeyHandler {
    fn handle(&self) -> io::Result<()> {
        // First, ensure we have a parent BLS key
        let parent_key_bytes = self.load_parent_key()?;

        // Gather session key parameters from user
        let namespace = self.get_namespace()?;
        let duration = self.get_duration()?;
        let purpose = self.get_purpose()?;

        // Confirm creation with user
        if !self.confirm_creation(&namespace, duration, &purpose)? {
            println!("Session key creation cancelled.");
            return Ok(());
        }

        // Create the session key
        match self
            .key_manager
            .create_session_key(&parent_key_bytes, &namespace, duration, &purpose)
        {
            Ok(session_data) => {
                self.display_session_key(&session_data);
                Ok(())
            }
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
        }
    }
}
