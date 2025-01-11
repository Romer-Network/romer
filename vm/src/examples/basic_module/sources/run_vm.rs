use anyhow::Result;
use romer_vm::{RomerVM, types::*, SuiPackageDeployer};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn main() -> Result<()> {
    // Create the VM instance
    println!("Creating VM instance...");
    let vm = Arc::new(Mutex::new(RomerVM::new()?));
    
    // Create the package deployer
    let deployer = SuiPackageDeployer::new(vm.clone());
    
    // Get path to our test module
    let package_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("basic_module");
    
    println!("Deploying package from: {:?}", package_path);
    
    // Deploy the module
    let deployed_modules = deployer.deploy_package(&package_path)?;
    println!("Successfully deployed {} modules", deployed_modules.len());
    
    // Get a handle to the VM to execute functions
    let vm_guard = vm.lock().unwrap();
    
    // Execute the increment function
    println!("Executing increment function...");
    vm_guard.execute_function(
        "basic",           // module name
        "increment",       // function name
        vec![],           // type arguments (empty for this example)
        vec![],           // arguments (empty for this example)
    )?;
    
    println!("Execution completed successfully!");
    Ok(())
}