// Basic trait that all handlers must implement
use std::io;

pub trait Handler {
    fn handle(&mut self) -> io::Result<()>;
}

// Declare the submodules
pub mod keymanager;
pub mod sequencer;


// Re-export the handlers from submodules for easier access
pub use keymanager::{
    CheckKeysHandler,
    CreateSessionKeyHandler, 
    GenerateKeypairHandler,
    SignMessageHandler
};

// FIX-related handler exports will go here as they are implemented
pub use sequencer::{
    LogonHandler,
    LogoutHandler,
    HeartbeatHandler,
    RegisterSenderCompIdHandler,
};
