//! Crankshaft TUI - Real-time monitoring dashboard
//! 
//! This application provides a terminal user interface for monitoring 
//! Crankshaft task execution across different backends.

use std::io;
use std::sync::{Arc, Mutex};
use color_eyre::Result;
use crankshaft_tui::app::{App, AppConfig};
use crankshaft_tui::event::EventHandler;
use crankshaft_tui::ui::Ui;
use crankshaft_tui::state::AppState; 

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    setup_terminal()?;
    
    // Create app configuration
    let config = AppConfig::new();

    // Create app instance and connect to Crankshaft engine
    let mut app = App::new(config).await?;

    // Create shared app state and UI components
    let app_state = Arc::new(Mutex::new(AppState::new()));
    let ui = Ui::new();

    // Setup event handler with state and UI
    let mut event_handler = EventHandler::new(app_state, ui);

    // Start the application loop with Crossterm backend
    app.run_with_crossterm(&mut event_handler).await?;

    // Restore terminal
    restore_terminal()?;
    
    Ok(())
}

fn setup_terminal() -> Result<()> {
    // Set up error handling
    color_eyre::install()?;
    
    // Setup logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    
    // Configure terminal
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        io::stdout(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    
    Ok(())
}

fn restore_terminal() -> Result<()> {
    // Restore terminal configuration
    crossterm::execute!(
        io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    crossterm::terminal::disable_raw_mode()?;
    
    Ok(())
}