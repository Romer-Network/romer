// Basic trait that all handlers must implement
use std::io;

pub trait Handler {
    fn handle(&self) -> io::Result<()>;
}

// Declare the submodules
pub mod keymanager;
pub mod fix;
pub mod sequencer;


// Re-export the handlers from submodules for easier access
pub use keymanager::{
    CheckKeysHandler,
    CreateSessionKeyHandler, 
    GenerateKeypairHandler,
    SignMessageHandler
};

// FIX-related handler exports will go here as they are implemented
pub use fix::{
    LogonHandler,
    LogoutHandler,
    HeartbeatHandler,
};

pub use sequencer::{
    StartSequencerHandler,
};