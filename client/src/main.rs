mod app;
mod handlers;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;

use crate::app::{App, Screen};

fn main() -> Result<()> {
    // Initialize terminal in raw mode and create alternate screen buffer
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    
    // Set up the terminal backend for ratatui
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create our application state
    let mut app = App::new();
    
    // Main event loop
    loop {
        // Draw the terminal user interface
        terminal.draw(|frame| {
            // Create the main layout with space for content and help text
            let size = frame.size();
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(3),  // Main content area
                    Constraint::Length(3), // Help text area
                ])
                .split(size);

            // Render different screens based on application state
            match app.screen {
                Screen::Welcome => {
                    // Create welcome screen items
                    let items = vec![
                        ListItem::new("Welcome to Rømer Chain Registration"),
                        ListItem::new(""),
                        ListItem::new("Select organization type:"),
                        ListItem::new("[M] Market Maker"),
                        ListItem::new("[S] Stablecoin Issuer"),
                    ];

                    // Create and render the welcome screen list
                    let list = List::new(items)
                        .block(Block::default()
                            .title("Registration")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Green)));

                    frame.render_widget(list, main_chunks[0]);
                }
                Screen::Registration => {
                    // Create layout for registration form fields
                    let input_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3),  // Organization name field
                            Constraint::Length(3),  // Sender Comp ID field
                            Constraint::Length(3),  // Success message area
                            Constraint::Min(0),     // Remaining space
                        ])
                        .split(main_chunks[0]);

                    // Organization name input field
                    let name_style = if app.selected_field == 0 {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default()
                    };

                    let name_input = Paragraph::new(Line::from(vec![
                        Span::raw("Organization Name: "),
                        Span::styled(&app.organization_name, name_style),
                        Span::raw(if app.selected_field == 0 { " ▋" } else { "" }),
                    ]))
                    .block(Block::default().borders(Borders::ALL));

                    // Sender Comp ID input field
                    let id_style = if app.selected_field == 1 {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default()
                    };

                    let id_input = Paragraph::new(Line::from(vec![
                        Span::raw("Sender Comp ID: "),
                        Span::styled(&app.sender_comp_id, id_style),
                        Span::raw(if app.selected_field == 1 { " ▋" } else { "" }),
                    ]))
                    .block(Block::default().borders(Borders::ALL));

                    // Render input fields
                    frame.render_widget(name_input, input_chunks[0]);
                    frame.render_widget(id_input, input_chunks[1]);

                    // Show success message if registration is complete
                    if app.show_success {
                        let success_msg = Paragraph::new("Registration successful!")
                            .style(Style::default().fg(Color::Green));
                        frame.render_widget(success_msg, input_chunks[2]);
                    }
                }
            }

            // Render help text at the bottom
            let help = Paragraph::new(app.get_help_text())
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(help, main_chunks[1]);

            // Show debug information in development
            #[cfg(debug_assertions)]
            {
                let debug_text = Paragraph::new(app.debug_state())
                    .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(debug_text, main_chunks[1]);
            }
        })?;

        // Handle keyboard input
        if let Event::Key(key) = event::read()? {
            // Only process key press events to avoid duplicate input
            if key.kind == KeyEventKind::Press {
                if handlers::handle_key(&mut app, key.code) {
                    break;
                }
            }
        }
    }

    // Restore terminal to original state
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}