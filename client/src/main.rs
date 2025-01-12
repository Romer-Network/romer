use std::io::{self, Write};

// Represents which menu we're currently displaying
enum CurrentMenu {
    Main,
    KeyManager,
    Fix,
    FixSequencer,
    FixSession,
    FixTrading,
    FixSettlement,
    Move,
}

fn main() {
    let mut current_menu = CurrentMenu::Main;
    
    loop {
        match current_menu {
            CurrentMenu::Main => {
                println!("\nMain Menu:");
                println!("1. KeyManager");
                println!("2. FIX");
                println!("3. Move");
                println!("4. Exit");
                
                match get_user_input().as_str() {
                    "1" => current_menu = CurrentMenu::KeyManager,
                    "2" => current_menu = CurrentMenu::Fix,
                    "3" => current_menu = CurrentMenu::Move,
                    "4" => break,
                    _ => println!("Invalid option, please try again"),
                }
            },
            
            CurrentMenu::KeyManager => {
                println!("\nKey Manager Menu:");
                println!("1. Check Existing Keys");
                println!("2. Generate KeyPair");
                println!("3. Sign a Message");
                println!("4. Create a Session Key");
                println!("5. Back to Main Menu");
                
                match get_user_input().as_str() {
                    "1" => println!("Check Existing Keys selected - functionality coming soon!"),
                    "2" => println!("Generate KeyPair selected - functionality coming soon!"),
                    "3" => println!("Sign a Message selected - functionality coming soon!"),
                    "4" => println!("Create a Session Key selected - functionality coming soon!"),
                    "5" => current_menu = CurrentMenu::Main,
                    _ => println!("Invalid option, please try again"),
                }
            },
            
            CurrentMenu::Fix => {
                println!("\nFIX Menu:");
                println!("1. Sequencer");
                println!("2. Session Management");
                println!("3. Trading");
                println!("4. Settlement");
                println!("5. Back to Main Menu");
                
                match get_user_input().as_str() {
                    "1" => current_menu = CurrentMenu::FixSequencer,
                    "2" => current_menu = CurrentMenu::FixSession,
                    "3" => current_menu = CurrentMenu::FixTrading,
                    "4" => current_menu = CurrentMenu::FixSettlement,
                    "5" => current_menu = CurrentMenu::Main,
                    _ => println!("Invalid option, please try again"),
                }
            },
            
            CurrentMenu::FixSequencer => {
                println!("\nSequencer Menu:");
                println!("1. Start Sequencer");
                println!("2. Simulate Block");
                println!("3. Back to FIX Menu");
                
                match get_user_input().as_str() {
                    "1" => println!("Start Sequencer selected - functionality coming soon!"),
                    "2" => println!("Simulate Block selected - functionality coming soon!"),
                    "3" => current_menu = CurrentMenu::Fix,
                    _ => println!("Invalid option, please try again"),
                }
            },
            
            CurrentMenu::FixSession => {
                println!("\nSession Management Menu:");
                println!("1. Logon");
                println!("2. Logout");
                println!("3. Heartbeat");
                println!("4. Back to FIX Menu");
                
                match get_user_input().as_str() {
                    "1" => println!("Logon selected - functionality coming soon!"),
                    "2" => println!("Logout selected - functionality coming soon!"),
                    "3" => println!("Heartbeat selected - functionality coming soon!"),
                    "4" => current_menu = CurrentMenu::Fix,
                    _ => println!("Invalid option, please try again"),
                }
            },
            
            CurrentMenu::FixTrading => {
                println!("\nTrading Menu:");
                println!("1. Order");
                println!("2. Back to FIX Menu");
                
                match get_user_input().as_str() {
                    "1" => println!("Order selected - functionality coming soon!"),
                    "2" => current_menu = CurrentMenu::Fix,
                    _ => println!("Invalid option, please try again"),
                }
            },
            
            CurrentMenu::FixSettlement => {
                println!("\nSettlement Menu:");
                println!("1. Settle");
                println!("2. Back to FIX Menu");
                
                match get_user_input().as_str() {
                    "1" => println!("Settle selected - functionality coming soon!"),
                    "2" => current_menu = CurrentMenu::Fix,
                    _ => println!("Invalid option, please try again"),
                }
            },
            
            CurrentMenu::Move => {
                println!("\nMove Menu:");
                println!("1. Compile Move Code");
                println!("2. Back to Main Menu");
                
                match get_user_input().as_str() {
                    "1" => println!("Compile Move Code selected - functionality coming soon!"),
                    "2" => current_menu = CurrentMenu::Main,
                    _ => println!("Invalid option, please try again"),
                }
            },
        }
    }
    
    println!("Goodbye!");
}

// Helper function to get user input
fn get_user_input() -> String {
    print!("> ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}