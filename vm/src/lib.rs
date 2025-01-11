//! Romer Move VM implementation optimized for trading operations.
//! 
//! This crate provides a Move VM implementation that focuses on efficient
//! execution of trading operations while maintaining compatibility with
//! Sui Move packages.

// Updated src/lib.rs
mod vm;
mod runtime;
mod natives;
mod storage;
mod verifier;
mod package;
mod error;

pub use vm::RomerVM;
pub use package::deployer::SuiPackageDeployer;

// Re-export common types that users of the VM will need
pub use crate::error::VMError;