// Updated src/vm.rs
use anyhow::Result;
use move_vm_runtime::move_vm::MoveVM;
use crate::{
    natives::table::build_natives,
    storage::modules::ModuleStore,
    runtime::session::SessionManager,
    error::VMError,
};

pub struct RomerVM {
    vm: MoveVM,
    module_store: ModuleStore,
    session_manager: SessionManager,
}

impl RomerVM {
    pub fn new() -> Result<Self, VMError> {
        let natives = build_natives();
        let vm = MoveVM::new(natives)
            .map_err(|e| VMError::Execution(e.to_string()))?;
            
        Ok(Self {
            vm,
            module_store: ModuleStore::new(),
            session_manager: SessionManager::new(),
        })
    }

    pub fn new_session(&self) -> Result<SessionManager, VMError> {
        self.session_manager.new_session(&self.vm, &self.module_store)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_creation() {
        let vm = RomerVM::new();
        assert!(vm.is_ok());
    }
}