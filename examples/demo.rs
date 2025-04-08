//! Crankshaft TUI demo application.
//!
//! This example demonstrates the Crankshaft TUI features with a simulated environment.
//! It shows how to:
//! 1. Set up the terminal
//! 2. Initialize the monitoring components
//! 3. Run the event loop
//! 4. Render the UI

use std::sync::{Arc, Mutex};
use std::error::Error;
use std::io;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use futures::StreamExt;

use crankshaft_tui::state::{AppState, TaskUpdate, BackendUpdate};
use crankshaft_tui::ui::Ui;
use crankshaft_tui::monitor::{TaskMonitor, BackendMonitor};
use crankshaft_tui::event::{Event, EventHandler, EventResult};
use rand::thread_rng;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Set up application state with mutex for thread safety
    let app_state = Arc::new(Mutex::new(AppState::new()));
    
    // Set up UI
    let ui = Ui::new();
    
    // Set up event handler with shared state
    let mut event_handler = EventHandler::new(Arc::clone(&app_state), ui);
    
    // Set up monitors
    let mut task_monitor = TaskMonitor::new();
    let mut backend_monitor = BackendMonitor::new();
    
    // Initialize monitors with simulated data
    task_monitor.connect("demo://localhost").await?;
    backend_monitor.connect("demo://localhost").await?;
    
    let mut _rng = thread_rng();
    
    // Run the event loop
    let mut should_exit = false;
    while !should_exit {
        // Draw the UI
        terminal.draw(|frame| {
            // Lock the state for reading
            if let Ok(state) = app_state.lock() {
                // Access a clone of the UI from event_handler
                event_handler.render(frame, &state);
            }
        })?;
        
        // Handle events
        if let Some(event) = event_handler.next().await {
            match event {
                Event::Key(_) => {
                    match event_handler.handle(event)? {
                        EventResult::Exit => should_exit = true,
                        EventResult::Continue => {},
                        EventResult::Ignored => {},
                    }
                },
                Event::Tick => {
                    // Update animations
                },
                Event::Resize(width, height) => {
                    // Handle terminal resize
                    if let Ok(mut state) = app_state.lock() {
                        state.terminal_width = width;
                        state.terminal_height = height;
                    }
                }
            }
        }
        
        // Poll for task updates
        if let Some(task_update) = task_monitor.poll().await {
            // Convert to state update type
            let state_update: TaskUpdate = task_update.into();
            
            // Update state
            if let Ok(mut state) = app_state.lock() {
                state.update_tasks(vec![state_update]);
            }
        }
        
        // Poll for backend updates
        if let Some(backend_update) = backend_monitor.poll().await {
            // Convert to state update type
            let state_update: BackendUpdate = backend_update.into();
            
            // Update state
            if let Ok(mut state) = app_state.lock() {
                state.update_backends(vec![state_update]);
            }
        }
    }
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    println!("Thank you for trying the Crankshaft TUI demo!");
    
    Ok(())
}

// Extension trait to add rendering method to EventHandler
trait EventHandlerExt {
    fn render(&self, frame: &mut ratatui::Frame, state: &AppState);
}

impl EventHandlerExt for EventHandler {
    fn render(&self, frame: &mut ratatui::Frame, state: &AppState) {
        // Create a dummy UI and call render on it with the provided state
        let ui = Ui::new();
        ui.render(frame, state);
    }
}