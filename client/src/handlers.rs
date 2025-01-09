use crossterm::event::KeyCode;
use crate::app::{App, Screen, OrgType};

pub fn handle_key(app: &mut App, key: KeyCode) -> bool {
    // The main key handler now lives in its own module
    match key {
        KeyCode::Char('q') | KeyCode::Esc => {
            // Handle escape/quit logic
            match app.screen {
                Screen::Welcome => return true, // Only quit from welcome screen
                Screen::Registration => {
                    // Return to welcome screen and reset state
                    app.screen = Screen::Welcome;
                    app.org_type = None;
                    app.organization_name.clear();
                    app.sender_comp_id.clear();
                }
            }
        }
        KeyCode::Char('m') | KeyCode::Char('M') => {
            // Handle market maker selection
            if app.screen == Screen::Welcome {
                app.org_type = Some(OrgType::MarketMaker);
                app.show_success = false;
                app.screen = Screen::Registration;
            }
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            // Handle stablecoin issuer selection
            if app.screen == Screen::Welcome {
                app.org_type = Some(OrgType::StablecoinIssuer);
                app.show_success = false;
                app.screen = Screen::Registration;
            }
        }
        KeyCode::Enter => {
            // Handle form submission
            if app.screen == Screen::Registration {
                if !app.organization_name.is_empty() && !app.sender_comp_id.is_empty() {
                    app.show_success = true;
                }
            }
        }
        KeyCode::Tab => {
            // Handle field navigation
            if app.screen == Screen::Registration {
                app.selected_field = (app.selected_field + 1) % 2;
            }
        }
        KeyCode::Backspace => {
            // Handle text deletion
            if app.screen == Screen::Registration {
                match app.selected_field {
                    0 => { app.organization_name.pop(); }
                    1 => { app.sender_comp_id.pop(); }
                    _ => {}
                }
            }
        }
        KeyCode::Char(c) => {
            // Handle text input
            if app.screen == Screen::Registration {
                match app.selected_field {
                    0 => app.organization_name.push(c),
                    1 => app.sender_comp_id.push(c),
                    _ => {}
                }
            }
        }
        _ => {}
    }
    false // Don't quit the application
}