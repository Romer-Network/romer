use super::state::{Session, SessionState, SessionError};
use crate::fix::types::ValidatedMessage;
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use dashmap::DashMap;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Manages all active FIX sessions for the sequencer
pub struct SessionManager {
    /// Active sessions indexed by session ID - using DashMap for thread-safe concurrent access
    sessions: DashMap<Uuid, Session>,
    /// Sessions indexed by sender comp ID for quick lookup during message processing
    sender_index: DashMap<String, Uuid>,
    /// Channel for forwarding validated messages to the batch manager
    message_tx: mpsc::Sender<ValidatedMessage>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(message_tx: mpsc::Sender<ValidatedMessage>) -> Self {
        Self {
            sessions: DashMap::new(),
            sender_index: DashMap::new(),
            message_tx,
        }
    }

    /// Start the session management background tasks
    pub async fn run(&self) {
        let mut interval = time::interval(Duration::from_secs(1));
        
        loop {
            interval.tick().await;
            self.check_sessions().await;
        }
    }

    /// Create a new session for a market maker
    /// Returns the session ID if successful
    pub fn create_session(
        &self,
        sender_comp_id: String,
        target_comp_id: String,
        heartbeat_interval: u32,
        public_key: Vec<u8>,
    ) -> Result<Uuid, SessionError> {
        // Check for existing session for this sender
        if let Some(existing_id) = self.sender_index.get(&sender_comp_id) {
            // Allow new session if the existing one is terminated
            if let Some(existing) = self.sessions.get(existing_id.value()) {
                if existing.state != SessionState::Terminated {
                    return Err(SessionError::AuthenticationFailed(
                        format!("Sender {} already has an active session", sender_comp_id)
                    ));
                }
                // Clean up terminated session
                self.sessions.remove(existing_id.value());
                self.sender_index.remove(&sender_comp_id);
            }
        }

        // Create and store new session
        let session = Session::new(
            sender_comp_id.clone(),
            target_comp_id,
            heartbeat_interval,
            public_key,
        );
        
        let session_id = session.session_id;
        
        // Store both primary and index references
        self.sessions.insert(session_id, session);
        self.sender_index.insert(sender_comp_id, session_id);
        
        info!(session_id = ?session_id, "Created new session");
        Ok(session_id)
    }

    /// Handle an incoming message for a specific session
    pub async fn handle_message(
        &self,
        session_id: Uuid,
        message: ValidatedMessage,
    ) -> Result<(), SessionError> {
        // Get and verify session exists
        let mut session = self.sessions.get_mut(&session_id)
            .ok_or_else(|| {
                error!(session_id = ?session_id, "Session not found");
                SessionError::NotFound(session_id)
            })?;
            
        // Verify session is in a state to accept messages
        match session.state {
            SessionState::Active => {},
            state => {
                error!(session_id = ?session_id, state = ?state, "Session not active");
                return Err(SessionError::InvalidState(state));
            }
        }

        // Update session sequence numbers and timing
        session.message_received(message.msg_seq_num)?;

        // Forward message for processing
        if let Err(e) = self.message_tx.send(message).await {
            error!(session_id = ?session_id, error = %e, "Failed to forward message");
            session.transition_to(SessionState::ResyncRequired)?;
            return Err(SessionError::ProcessingFailed(e.to_string()));
        }

        Ok(())
    }

    /// Periodic check of all active sessions
    async fn check_sessions(&self) {
        let mut heartbeat_needed = Vec::new();
        let mut timeouts = Vec::new();

        // First pass: identify sessions needing attention
        for session in self.sessions.iter() {
            if session.state != SessionState::Active {
                continue;
            }

            if session.is_heartbeat_overdue() {
                timeouts.push(session.session_id);
            } else if session.needs_heartbeat() {
                heartbeat_needed.push(session.session_id);
            }
        }

        // Handle heartbeats
        for session_id in heartbeat_needed {
            if let Some(mut session) = self.sessions.get_mut(&session_id) {
                if let Err(e) = self.send_heartbeat(&mut session).await {
                    error!(session_id = ?session_id, error = %e, "Failed to send heartbeat");
                }
            }
        }

        // Handle timeouts
        for session_id in timeouts {
            if let Some(mut session) = self.sessions.get_mut(&session_id) {
                warn!(session_id = ?session_id, "Session timed out, terminating");
                if let Err(e) = self.terminate_session_internal(&mut session).await {
                    error!(session_id = ?session_id, error = %e, "Failed to terminate session");
                }
            }
        }
    }

    /// Send a heartbeat message for a session
    async fn send_heartbeat(&self, session: &mut Session) -> Result<(), SessionError> {
        // Create heartbeat message
        let heartbeat = self.create_heartbeat_message(session)?;
        
        // Update session state
        session.message_sent();
        
        // Send through normal message path
        self.message_tx.send(heartbeat).await
            .map_err(|e| SessionError::ProcessingFailed(e.to_string()))?;
            
        Ok(())
    }

    /// Create a FIX heartbeat message
    fn create_heartbeat_message(&self, session: &Session) -> Result<ValidatedMessage, SessionError> {
        // TODO: Implement actual FIX heartbeat message creation
        // For now returning placeholder
        unimplemented!("Heartbeat message creation not implemented")
    }

    /// Internal method to terminate a session
    async fn terminate_session_internal(&self, session: &mut Session) -> Result<(), SessionError> {
        // Transition through proper states
        session.transition_to(SessionState::Disconnecting)?;
        session.transition_to(SessionState::Terminated)?;
        
        // Remove from sender index
        self.sender_index.remove(&session.sender_comp_id);
        
        info!(session_id = ?session.session_id, "Session terminated");
        Ok(())
    }

    /// Gracefully terminate a session
    pub async fn terminate_session(&self, session_id: Uuid) -> Result<(), SessionError> {
        let mut session = self.sessions.get_mut(&session_id)
            .ok_or(SessionError::NotFound(session_id))?;
            
        self.terminate_session_internal(&mut session).await
    }

    /// Get information about a specific session
    pub fn get_session(&self, session_id: Uuid) -> Result<Session, SessionError> {
        self.sessions.get(&session_id)
            .map(|s| s.value().clone())
            .ok_or(SessionError::NotFound(session_id))
    }

    /// Get current active session count
    pub fn active_session_count(&self) -> usize {
        self.sessions.iter()
            .filter(|s| s.state == SessionState::Active)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_session_lifecycle() {
        let (tx, _rx) = mpsc::channel(100);
        let manager = SessionManager::new(tx);

        // Create session
        let session_id = manager.create_session(
            "SENDER".to_string(),
            "TARGET".to_string(),
            30,
            vec![1, 2, 3, 4],
        ).unwrap();

        // Verify session exists
        let session = manager.get_session(session_id).unwrap();
        assert_eq!(session.state, SessionState::Connecting);

        // Terminate session
        manager.terminate_session(session_id).await.unwrap();
        
        // Verify session is terminated
        let session = manager.get_session(session_id).unwrap();
        assert_eq!(session.state, SessionState::Terminated);
    }

    #[tokio::test]
    async fn test_duplicate_session_prevention() {
        let (tx, _rx) = mpsc::channel(100);
        let manager = SessionManager::new(tx);

        // Create first session
        manager.create_session(
            "SENDER".to_string(),
            "TARGET".to_string(),
            30,
            vec![1, 2, 3, 4],
        ).unwrap();

        // Try to create duplicate session
        let result = manager.create_session(
            "SENDER".to_string(),
            "TARGET".to_string(),
            30,
            vec![1, 2, 3, 4],
        );

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_session_timeout() {
        let (tx, _rx) = mpsc::channel(100);
        let manager = SessionManager::new(tx);

        // Create and start manager
        let manager_clone = manager.clone();
        tokio::spawn(async move {
            manager_clone.run().await;
        });

        // Create session
        let session_id = manager.create_session(
            "SENDER".to_string(),
            "TARGET".to_string(),
            1, // 1 second heartbeat for faster testing
            vec![1, 2, 3, 4],
        ).unwrap();

        // Wait for timeout
        sleep(Duration::from_secs(3)).await;

        // Verify session was terminated
        let session = manager.get_session(session_id).unwrap();
        assert_eq!(session.state, SessionState::Terminated);
    }
}