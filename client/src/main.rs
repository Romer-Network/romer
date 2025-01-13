mod handlers;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use handlers::{
    CheckKeysHandler, CreateSessionKeyHandler, GenerateKeypairHandler, Handler, HeartbeatHandler,
    LogonHandler, LogoutHandler, SignMessageHandler, StartSequencerHandler,
};
use std::io::{self, stdout, Write};

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

// Helper function to clear the screen and reset cursor position
fn clear_screen() -> io::Result<()> {
    stdout().execute(Clear(ClearType::All))?;
    // Move cursor to top-left corner after clearing
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush()?;
    Ok(())
}

// Modified input function to handle ESC key
fn get_user_input() -> io::Result<Option<String>> {
    print!("> ");
    io::stdout().flush()?;

    // Enable raw mode to read individual keystrokes
    crossterm::terminal::enable_raw_mode()?;

    let result = loop {
        // Wait for a key event
        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                // Handle the ESC key
                KeyCode::Esc => {
                    crossterm::terminal::disable_raw_mode()?;
                    return Ok(None);
                }
                // Handle the Enter key
                KeyCode::Enter => {
                    println!(); // Move to next line
                    break Ok(Some(String::new()));
                }
                // Handle regular characters
                KeyCode::Char(c) => {
                    print!("{}", c);
                    io::stdout().flush()?;
                    break Ok(Some(c.to_string()));
                }
                _ => continue,
            }
        }
    };

    // Disable raw mode after input
    crossterm::terminal::disable_raw_mode()?;
    result
}

fn main() -> io::Result<()> {
    let mut current_menu = CurrentMenu::Main;

    // Clear screen at startup
    clear_screen()?;

    loop {
        match current_menu {
            CurrentMenu::Main => {
                println!("\nMain Menu:");
                println!("1. KeyManager");
                println!("2. FIX");
                println!("3. Move");
                println!("4. Exit");
                println!("\nPress ESC at any time to return to the previous menu");

                match get_user_input()? {
                    Some(input) => match input.as_str() {
                        "1" => {
                            current_menu = CurrentMenu::KeyManager;
                            clear_screen()?;
                        }
                        "2" => {
                            current_menu = CurrentMenu::Fix;
                            clear_screen()?;
                        }
                        "3" => {
                            current_menu = CurrentMenu::Move;
                            clear_screen()?;
                        }
                        "4" => break,
                        _ => println!("Invalid option, please try again"),
                    },
                    None => continue, // ESC pressed, stay in current menu
                }
            }

            CurrentMenu::KeyManager => {
                println!("\nKey Manager Menu:");
                println!("1. Check Existing Keys");
                println!("2. Generate KeyPair");
                println!("3. Sign a Message");
                println!("4. Create a Session Key");
                println!("5. Back to Main Menu");
                println!("\nPress ESC at any time to return to the previous menu");

                match get_user_input()? {
                    Some(input) => match input.as_str() {
                        "1" => match CheckKeysHandler::new() {
                            Ok(handler) => {
                                if let Err(e) = handler.handle() {
                                    println!("Error checking keys: {}", e);
                                }
                                println!("\nPress Enter to continue...");
                                get_user_input()?;
                                clear_screen()?;
                            }
                            Err(e) => println!("Error creating key manager: {}", e),
                        },
                        "2" => match GenerateKeypairHandler::new() {
                            Ok(handler) => {
                                if let Err(e) = handler.handle() {
                                    println!("Error generating keypair: {}", e);
                                }
                                println!("\nPress Enter to continue...");
                                get_user_input()?;
                                clear_screen()?;
                            }
                            Err(e) => println!("Error creating key manager: {}", e),
                        },
                        "3" => match SignMessageHandler::new() {
                            Ok(handler) => {
                                if let Err(e) = handler.handle() {
                                    println!("Error signing message: {}", e);
                                }
                                println!("\nPress Enter to continue...");
                                get_user_input()?;
                                clear_screen()?;
                            }
                            Err(e) => println!("Error creating key manager: {}", e),
                        },
                        "4" => match CreateSessionKeyHandler::new() {
                            Ok(handler) => {
                                if let Err(e) = handler.handle() {
                                    println!("Error creating session key: {}", e);
                                }
                                println!("\nPress Enter to continue...");
                                get_user_input()?;
                                clear_screen()?;
                            }
                            Err(e) => println!("Error creating key manager: {}", e),
                        },
                        "5" => {
                            current_menu = CurrentMenu::Main;
                            clear_screen()?;
                        }
                        _ => println!("Invalid option, please try again"),
                    },
                    None => {
                        current_menu = CurrentMenu::Main;
                        clear_screen()?;
                    }
                }
            }

            CurrentMenu::Fix => {
                println!("\nFIX Menu:");
                println!("1. Sequencer");
                println!("2. Session Management");
                println!("3. Trading");
                println!("4. Settlement");
                println!("5. Back to Main Menu");
                println!("\nPress ESC at any time to return to the previous menu");

                match get_user_input()? {
                    Some(input) => match input.as_str() {
                        "1" => {
                            current_menu = CurrentMenu::FixSequencer;
                            clear_screen()?;
                        }
                        "2" => {
                            current_menu = CurrentMenu::FixSession;
                            clear_screen()?;
                        }
                        "3" => {
                            current_menu = CurrentMenu::FixTrading;
                            clear_screen()?;
                        }
                        "4" => {
                            current_menu = CurrentMenu::FixSettlement;
                            clear_screen()?;
                        }
                        "5" => {
                            current_menu = CurrentMenu::Main;
                            clear_screen()?;
                        }
                        _ => println!("Invalid option, please try again"),
                    },
                    None => {
                        current_menu = CurrentMenu::Main;
                        clear_screen()?;
                    }
                }
            }

            CurrentMenu::FixSequencer => {
                println!("\nSequencer Menu:");
                println!("1. Start Sequencer");
                println!("2. Simulate Block");
                println!("3. Back to FIX Menu");
                println!("\nPress ESC at any time to return to the previous menu");

                match get_user_input()? {
                    Some(input) => match input.as_str() {
                        "1" => match StartSequencerHandler::new() {
                            Ok(handler) => {
                                // Start the sequencer and handle any errors
                                if let Err(e) = handler.handle() {
                                    println!("Error starting sequencer: {}", e);
                                }
                                // Wait for user acknowledgment before clearing screen
                                println!("\nPress Enter to continue...");
                                get_user_input()?;
                                clear_screen()?;
                            }
                            Err(e) => {
                                // Handle any initialization errors
                                println!("Error creating sequencer handler: {}", e);
                                println!("\nPress Enter to continue...");
                                get_user_input()?;
                                clear_screen()?;
                            }
                        },
                        "2" => {
                            println!("Simulate Block selected - functionality coming soon!");
                            println!("\nPress Enter to continue...");
                            get_user_input()?;
                            clear_screen()?;
                        }
                        "3" => {
                            current_menu = CurrentMenu::Fix;
                            clear_screen()?;
                        }
                        _ => println!("Invalid option, please try again"),
                    },
                    None => {
                        current_menu = CurrentMenu::Fix;
                        clear_screen()?;
                    }
                }
            }

            CurrentMenu::FixSession => {
                println!("\nSession Management Menu:");
                println!("1. Logon");
                println!("2. Logout");
                println!("3. Heartbeat");
                println!("4. Back to FIX Menu");
                println!("\nPress ESC at any time to return to the previous menu");

                match get_user_input()? {
                    Some(input) => match input.as_str() {
                        "1" => match LogonHandler::new() {
                            Ok(handler) => {
                                if let Err(e) = handler.handle() {
                                    println!("Error handling logon: {}", e);
                                }
                                println!("\nPress Enter to continue...");
                                get_user_input()?;
                                clear_screen()?;
                            }
                            Err(e) => println!("Error creating logon handler: {}", e),
                        },
                        "2" => {
                            let handler = LogoutHandler::new();
                            if let Err(e) = handler.handle() {
                                println!("Error handling logout: {}", e);
                            }
                            println!("\nPress Enter to continue...");
                            get_user_input()?;
                            clear_screen()?;
                        }
                        "3" => {
                            let handler = HeartbeatHandler::new();
                            if let Err(e) = handler.handle() {
                                println!("Error handling heartbeat: {}", e);
                            }
                            println!("\nPress Enter to continue...");
                            get_user_input()?;
                            clear_screen()?;
                        }
                        "4" => {
                            current_menu = CurrentMenu::Fix;
                            clear_screen()?;
                        }
                        _ => println!("Invalid option, please try again"),
                    },
                    None => {
                        current_menu = CurrentMenu::Fix;
                        clear_screen()?;
                    }
                }
            }

            CurrentMenu::FixTrading => {
                println!("\nTrading Menu:");
                println!("1. Order");
                println!("2. Back to FIX Menu");
                println!("\nPress ESC at any time to return to the previous menu");

                match get_user_input()? {
                    Some(input) => match input.as_str() {
                        "1" => {
                            println!("Order selected - functionality coming soon!");
                            println!("\nPress Enter to continue...");
                            get_user_input()?;
                            clear_screen()?;
                        }
                        "2" => {
                            current_menu = CurrentMenu::Fix;
                            clear_screen()?;
                        }
                        _ => println!("Invalid option, please try again"),
                    },
                    None => {
                        current_menu = CurrentMenu::Fix;
                        clear_screen()?;
                    }
                }
            }

            CurrentMenu::FixSettlement => {
                println!("\nSettlement Menu:");
                println!("1. Settle");
                println!("2. Back to FIX Menu");
                println!("\nPress ESC at any time to return to the previous menu");

                match get_user_input()? {
                    Some(input) => match input.as_str() {
                        "1" => {
                            println!("Settle selected - functionality coming soon!");
                            println!("\nPress Enter to continue...");
                            get_user_input()?;
                            clear_screen()?;
                        }
                        "2" => {
                            current_menu = CurrentMenu::Fix;
                            clear_screen()?;
                        }
                        _ => println!("Invalid option, please try again"),
                    },
                    None => {
                        current_menu = CurrentMenu::Fix;
                        clear_screen()?;
                    }
                }
            }

            CurrentMenu::Move => {
                println!("\nMove Menu:");
                println!("1. Compile Move Code");
                println!("2. Back to Main Menu");
                println!("\nPress ESC at any time to return to the previous menu");

                match get_user_input()? {
                    Some(input) => match input.as_str() {
                        "1" => {
                            println!("Compile Move Code selected - functionality coming soon!");
                            println!("\nPress Enter to continue...");
                            get_user_input()?;
                            clear_screen()?;
                        }
                        "2" => {
                            current_menu = CurrentMenu::Main;
                            clear_screen()?;
                        }
                        _ => println!("Invalid option, please try again"),
                    },
                    None => {
                        current_menu = CurrentMenu::Main;
                        clear_screen()?;
                    }
                }
            }
        }
    }

    // Clear screen before exiting
    clear_screen()?;
    println!("Goodbye!");
    Ok(())
}
