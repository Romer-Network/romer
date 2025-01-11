// src/storage/modules.rs
use move_binary_format::CompiledModule;
use move_core_types::language_storage::ModuleId;
use std::collections::HashMap;
use crate::error::VMError;

/// Stores and manages deployed Move modules
pub struct ModuleStore {
    /// Maps module IDs to their compiled bytecode
    modules: HashMap<ModuleId, Vec<u8>>,
}

impl ModuleStore {
    /// Create a new empty module store
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    /// Store a new module, deserializing it first to verify its correctness
    /// and extract the module ID
    pub fn store_module(&mut self, module_bytes: Vec<u8>) -> Result<ModuleId, VMError> {
        // First, attempt to deserialize the module using the recommended method
        // This will validate that the bytecode is well-formed
        let module = CompiledModule::deserialize_with_defaults(&module_bytes)
            .map_err(|e| VMError::ModuleDeployment(format!("Failed to deserialize module: {}", e)))?;
            
        // Extract the module's ID - this uniquely identifies the module
        let module_id = module.self_id();
        
        // Store the original bytecode - we keep the original bytes rather than 
        // re-serializing the deserialized module to preserve exact byte-for-byte compatibility
        self.modules.insert(module_id.clone(), module_bytes);
        
        Ok(module_id)
    }

    /// Retrieve a module's bytecode by its ID
    pub fn get_module(&self, id: &ModuleId) -> Option<&Vec<u8>> {
        self.modules.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_module_storage() {
        let mut store = ModuleStore::new();
        // Add test implementation here once we have sample Move modules
    }
}