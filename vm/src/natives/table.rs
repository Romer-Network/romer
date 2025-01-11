// src/natives/table.rs
use move_vm_runtime::native_functions::NativeFunctionTable;

pub fn build_natives() -> NativeFunctionTable {
    // Start with an empty native function table
    NativeFunctionTable::new()
}