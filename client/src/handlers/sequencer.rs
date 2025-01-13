use crate::handlers::Handler;
use std::io::{self, Write};
use std::process::{Child, Command};
use tracing::{error, info};

pub struct StartSequencerHandler {
    // Track the running sequencer process
    process: Option<Child>,
    // Default configuration
    host: String,
    port: u16,
}

impl StartSequencerHandler {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            process: None,
            host: "127.0.0.1".to_string(),
            port: 9878,
        })
    }

    // Get configuration from user input
    fn get_config(&self) -> io::Result<(String, u16)> {
        println!("\nSequencer Configuration");
        println!("Enter values or press Enter for defaults:");

        print!("Host [{}]: ", self.host);
        io::stdout().flush()?;
        let mut host = String::new();
        io::stdin().read_line(&mut host)?;
        let host = host.trim();

        print!("Port [{}]: ", self.port);
        io::stdout().flush()?;
        let mut port = String::new();
        io::stdin().read_line(&mut port)?;
        let port = port.trim();

        Ok((
            if host.is_empty() {
                self.host.clone()
            } else {
                host.to_string()
            },
            if port.is_empty() {
                self.port
            } else {
                port.parse().unwrap_or(self.port)
            },
        ))
    }

    // Start the sequencer as a child process
    fn start_sequencer(&mut self, host: String, port: u16) -> io::Result<()> {
        let mut command = Command::new("cargo");

        command
            .args(["run", "--bin", "romer-sequencer"]) // Updated binary name
            .env("SEQUENCER_HOST", host)
            .env("SEQUENCER_PORT", port.to_string());

        // Start the process
        let child = command.spawn()?;

        // Store the process handle so we can manage it
        self.process = Some(child);

        Ok(())
    }

    // Display the sequencer status
    fn display_status(&self, host: &str, port: u16) {
        println!("\nSequencer Status:");
        println!("----------------");
        println!("Status: Starting");
        println!("Host: {}", host);
        println!("Port: {}", port);
        println!("\nStarting sequencer process...");
        println!("Use the sequencer logs to monitor startup progress.");
        println!("Press Ctrl+C to stop the sequencer when done.");
    }
}

impl Handler for StartSequencerHandler {
    fn handle(&self) -> io::Result<()> {
        // Get configuration from user
        let (host, port) = self.get_config()?;

        // Show initial status
        self.display_status(&host, port);

        // Create a mutable copy of self since we need to modify process
        let mut handler = StartSequencerHandler::new()?;

        // Start the sequencer process
        handler.start_sequencer(host, port)?;

        println!("\nSequencer started successfully!");
        println!("Process is running in the background.");

        Ok(())
    }
}
