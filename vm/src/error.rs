// src/error.rs
use thiserror::Error;
use std::error;

#[derive(Error, Debug)]
pub enum VMError {
    #[error("Module deployment failed: {0}")]
    ModuleDeployment(String),
    
    #[error("Execution failed: {0}")]
    Execution(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Verification failed: {0}")]
    Verification(String),

    #[error(transparent)]
    Common(#[from] Box<dyn error::Error + Send + Sync>),
}