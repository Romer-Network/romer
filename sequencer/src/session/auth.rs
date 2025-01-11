use super::state::{Session, SessionState, SessionError};
use fefix::prelude::*;
use fefix::tagvalue::SetGetField;
use blst::min_pk::{SecretKey, PublicKey, Signature};
use sha2::{Sha256, Digest};
use hex;
use tracing::{info, warn, error};

/// Handles authentication for FIX sessions using BLS signatures
pub struct SessionAuthenticator {
    /// Registry of known public keys indexed by sender comp ID
    registered_keys: dashmap::DashMap<String, PublicKey>,
}

impl SessionAuthenticator {
    pub fn new() -> Self {
        Self {
            registered_keys: dashmap::DashMap::new(),
        }
    }

    /// Register a new market maker's public key
    pub fn register_key(&self, sender_comp_id: String, public_key: &[u8]) -> Result<(), AuthError> {
        // Verify key format
        let pk = PublicKey::from_bytes(public_key)
            .map_err(|_| AuthError::InvalidPublicKey("Invalid public key format".to_string()))?;

        // Store the key
        self.registered_keys.insert(sender_comp_id, pk);
        Ok(())
    }

    /// Authenticate a logon message using BLS signature
    pub fn authenticate_logon(
        &self,
        session: &mut Session,
        message: &fefix::tagvalue::Message,
    ) -> Result<(), AuthError> {
        // Verify session is in correct state
        if session.state != SessionState::Authenticating {
            return Err(AuthError::InvalidState(
                "Session must be in Authenticating state".to_string()
            ));
        }

        // Extract authentication data from logon message
        let sender_comp_id = message.get_field::<SenderCompID>()
            .map_err(|_| AuthError::MissingField("SenderCompID".to_string()))?
            .as_str();

        let signature_hex = message.get_field::<Password>()
            .map_err(|_| AuthError::MissingField("Password (Signature)".to_string()))?
            .as_str();

        // Get registered public key
        let public_key = self.registered_keys.get(sender_comp_id)
            .ok_or_else(|| AuthError::UnknownSender(sender_comp_id.to_string()))?;

        // Verify the signature
        if !self.verify_signature(
            sender_comp_id,
            signature_hex,
            &public_key,
            message,
        )? {
            return Err(AuthError::InvalidSignature("Signature verification failed".to_string()));
        }

        // Update session state
        session.transition_to(SessionState::Active)
            .map_err(|e| AuthError::SessionError(e))?;

        info!(
            session_id = ?session.session_id,
            sender = sender_comp_id,
            "Session authenticated successfully"
        );

        Ok(())
    }

    /// Verify a BLS signature on a logon message
    fn verify_signature(
        &self,
        sender_comp_id: &str,
        signature_hex: &str,
        public_key: &PublicKey,
        message: &fefix::tagvalue::Message,
    ) -> Result<bool, AuthError> {
        // Decode the hex signature
        let signature_bytes = hex::decode(signature_hex)
            .map_err(|_| AuthError::InvalidSignature("Invalid signature format".to_string()))?;

        let signature = Signature::from_bytes(&signature_bytes)
            .map_err(|_| AuthError::InvalidSignature("Invalid signature bytes".to_string()))?;

        // Create message hash for verification
        // We hash specific fields from the logon message to create the signed content
        let msg_hash = self.create_logon_hash(message)?;

        // Verify the signature
        Ok(signature.verify(true, &msg_hash, &[], &public_key))
    }

    /// Create a hash of the logon message fields that were signed
    fn create_logon_hash(&self, message: &fefix::tagvalue::Message) -> Result<[u8; 32], AuthError> {
        let mut hasher = Sha256::new();

        // Add required fields to the hash in a deterministic order
        let fields = [
            ("SenderCompID", true),
            ("TargetCompID", true),
            ("SendingTime", true),
            ("HeartBtInt", true),
            ("EncryptMethod", false),  // Optional
            ("RawData", false),        // Optional
        ];

        for (field_name, required) in fields.iter() {
            match message.get_field_by_tag(field_name) {
                Ok(value) => {
                    hasher.update(field_name.as_bytes());
                    hasher.update(b"=");
                    hasher.update(value.as_str().as_bytes());
                    hasher.update(b"|");
                },
                Err(_) if !required => continue,
                Err(_) => return Err(AuthError::MissingField(field_name.to_string())),
            }
        }

        Ok(hasher.finalize().into())
    }
}

/// Errors that can occur during authentication
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Unknown sender: {0}")]
    UnknownSender(String),

    #[error("Invalid session state: {0}")]
    InvalidState(String),

    #[error("Session error: {0}")]
    SessionError(#[from] SessionError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use blst::min_pk::{SecretKey, PublicKey};
    use rand::{thread_rng, RngCore};

    fn create_test_keypair() -> (SecretKey, PublicKey) {
        let mut rng = thread_rng();
        let mut ikm = [0u8; 32];
        rng.fill_bytes(&mut ikm);
        
        let sk = SecretKey::key_gen(&ikm, &[]).unwrap();
        let pk = PublicKey::from_secret_key(&sk);
        
        (sk, pk)
    }

    fn create_test_logon_message() -> fefix::tagvalue::Message {
        let mut msg = fefix::tagvalue::Message::new(fefix::Dictionary::fix42());
        // Add required fields...
        msg
    }

    #[test]
    fn test_key_registration() {
        let authenticator = SessionAuthenticator::new();
        let (_sk, pk) = create_test_keypair();

        let result = authenticator.register_key(
            "SENDER".to_string(),
            pk.to_bytes().as_ref(),
        );
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_authentication_flow() {
        let authenticator = SessionAuthenticator::new();
        let (sk, pk) = create_test_keypair();

        // Register the key
        authenticator.register_key(
            "SENDER".to_string(),
            pk.to_bytes().as_ref(),
        ).unwrap();

        // Create a session
        let mut session = Session::new(
            "SENDER".to_string(),
            "TARGET".to_string(),
            30,
            pk.to_bytes().to_vec(),
        );
        
        session.transition_to(SessionState::Authenticating).unwrap();

        // Create and sign a logon message
        let msg = create_test_logon_message();
        let hash = authenticator.create_logon_hash(&msg).unwrap();
        let sig = sk.sign(&hash, &[], &[]);

        // Add signature to message
        // In real implementation, add signature to Password field
        
        // Verify authentication
        let result = authenticator.authenticate_logon(&mut session, &msg);
        assert!(result.is_ok());
        assert_eq!(session.state, SessionState::Active);
    }
}