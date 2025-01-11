// src/session/state.rs

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::time::Duration;
use uuid::Uuid;

/// Represents the current state of a FIX session
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SessionState {
    /// Initial connection being established
    Connecting,
    /// Authenticating the market maker
    Authenticating,
    /// Session is active and can process messages
    Active,
    /// Session is active but waiting for sequence reset
    ResyncRequired,
    /// Session is being gracefully closed
    Disconnecting,
    /// Session has been terminated
    Terminated,
}

/// Contains all the information about a FIX session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique identifier for this session
    pub session_id: Uuid,
    /// The market maker's sender comp ID
    pub sender_comp_id: String,
    /// Our target comp ID
    pub target_comp_id: String,
    /// Current state of the session
    pub state: SessionState,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// Last time a message was received
    pub last_received: DateTime<Utc>,
    /// Last time a message was sent
    pub last_sent: DateTime<Utc>,
    /// Next expected incoming message sequence number
    pub next_incoming_seq: u64,
    /// Next outgoing message sequence number
    pub next_outgoing_seq: u64,
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u32,
    /// Market maker's BLS public key
    pub public_key: Vec<u8>,
}

impl Session {
    /// Create a new session
    pub fn new(
        sender_comp_id: String,
        target_comp_id: String,
        heartbeat_interval: u32,
        public_key: Vec<u8>,
    ) -> Self {
        let now = Utc::now();
        Self {
            session_id: Uuid::new_v4(),
            sender_comp_id,
            target_comp_id,
            state: SessionState::Connecting,
            created_at: now,
            last_received: now,
            last_sent: now,
            next_incoming_seq: 1,
            next_outgoing_seq: 1,
            heartbeat_interval,
            public_key,
        }
    }

    /// Check if heartbeat is overdue
    pub fn is_heartbeat_overdue(&self) -> bool {
        let elapsed = Utc::now() - self.last_received;
        elapsed > Duration::from_secs(self.heartbeat_interval as u64 + 1)
    }

    /// Update the last received time and sequence number
    pub fn message_received(&mut self, seq_num: u64) -> Result<(), SessionError> {
        // Verify sequence number
        if seq_num != self.next_incoming_seq {
            return Err(SessionError::InvalidSequence {
                expected: self.next_incoming_seq,
                received: seq_num,
            });
        }

        self.last_received = Utc::now();
        self.next_incoming_seq += 1;
        Ok(())
    }

    /// Update the last sent time and sequence number
    pub fn message_sent(&mut self) {
        self.last_sent = Utc::now();
        self.next_outgoing_seq += 1;
    }

    /// Check if this session needs a heartbeat sent
    pub fn needs_heartbeat(&self) -> bool {
        let elapsed = Utc::now() - self.last_sent;
        elapsed >= Duration::from_secs((self.heartbeat_interval as f64 * 0.7) as u64)
    }

    /// Transition the session state
    pub fn transition_to(&mut self, new_state: SessionState) -> Result<(), SessionError> {
        use SessionState::*;
        
        // Validate state transition
        match (self.state, new_state) {
            // Valid transitions
            (Connecting, Authenticating) |
            (Authenticating, Active) |
            (Active, ResyncRequired) |
            (ResyncRequired, Active) |
            (Active, Disconnecting) |
            (Disconnecting, Terminated) => {
                self.state = new_state;
                Ok(())
            },
            // Invalid transitions
            (current, new) => Err(SessionError::InvalidTransition {
                from: current,
                to: new,
            }),
        }
    }
}

/// Errors that can occur during session operations
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Invalid sequence number (expected {expected}, received {received})")]
    InvalidSequence {
        expected: u64,
        received: u64,
    },

    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: SessionState,
        to: SessionState,
    },

    #[error("Session not found: {0}")]
    NotFound(Uuid),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_session() -> Session {
        Session::new(
            "SENDER".to_string(),
            "TARGET".to_string(),
            30,
            vec![1, 2, 3, 4], // Dummy public key
        )
    }

    #[test]
    fn test_session_creation() {
        let session = create_test_session();
        assert_eq!(session.state, SessionState::Connecting);
        assert_eq!(session.next_incoming_seq, 1);
        assert_eq!(session.next_outgoing_seq, 1);
    }

    #[test]
    fn test_sequence_tracking() {
        let mut session = create_test_session();
        
        // Test valid sequence
        assert!(session.message_received(1).is_ok());
        assert_eq!(session.next_incoming_seq, 2);

        // Test invalid sequence
        assert!(session.message_received(3).is_err());
    }

    #[test]
    fn test_state_transitions() {
        let mut session = create_test_session();
        
        // Test valid transitions
        assert!(session.transition_to(SessionState::Authenticating).is_ok());
        assert!(session.transition_to(SessionState::Active).is_ok());
        
        // Test invalid transition
        assert!(session.transition_to(SessionState::Connecting).is_err());
    }
}