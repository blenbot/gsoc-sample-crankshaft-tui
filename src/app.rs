//! Application state and logic.
//!
//! This module contains the main application state and handles the
//! integration between Crankshaft engine, UI components, and event handling.

use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::backend::Backend;
use ratatui::Terminal;
use crossterm::event::KeyCode;
use crate::event::{Event, EventHandler};
use crate::monitor::{TaskMonitor, BackendMonitor};
use crate::state::{AppState, Temporality};
use crate::ui::{self, Ui};

use futures::StreamExt;

/// Application configuration.
pub struct AppConfig {
    pub tick_rate_ms: u64,
    pub refresh_rate_ms: u64,
    pub debug_mode: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            tick_rate_ms: 250,
            refresh_rate_ms: 1000,
            debug_mode: false,
        }
    }
}

impl AppConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Main application.
pub struct App {
    /// Application state
    state: AppState,
    /// Application configuration
    config: AppConfig,
    /// Task monitor
    task_monitor: TaskMonitor,
    /// Backend monitor
    backend_monitor: BackendMonitor,
    /// Current view controller
    ui: Ui,
    /// Should the application exit?
    should_quit: bool,
}

impl App {
    /// Creates a new application instance.
    pub async fn new(config: AppConfig) -> Result<Self> {
        // Initialize app state with the Entity-Component pattern from tokio-console
        let state = AppState::new();
        
        // Initialize monitors for crankshaft engine
        let task_monitor = TaskMonitor::new();
        let backend_monitor = BackendMonitor::new();
        
        // Initialize UI controller
        let ui = Ui::new();
        
        Ok(Self {
            state,
            config,
            task_monitor,
            backend_monitor,
            ui,
            should_quit: false,
        })
    }
    
    /// Runs the application main loop.
    pub async fn run(&mut self, terminal: &mut Terminal<impl Backend>, event_handler: &mut EventHandler) -> Result<()> {
        // Main loop
        while !self.should_quit {
            // Draw the UI
            terminal.draw(|frame| self.ui.render(frame, &self.state))?;
            
            // Handle events
            if let Some(event) = event_handler.next().await {
                self.handle_event(event)?;
            }
            
            // Update state
            self.update().await?;
        }
        
        Ok(())
    }
    
    /// Runs the application main loop with Crossterm backend.
    pub async fn run_with_crossterm(&mut self, event_handler: &mut EventHandler) -> Result<()> {
        use ratatui::backend::CrosstermBackend;
        
        // Create terminal
        let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        
        // Main loop
        while !self.should_quit {
            // Draw the UI
            terminal.draw(|frame| self.ui.render(frame, &self.state))?;
            
            // Handle events
            if let Some(event) = event_handler.next().await {
                self.handle_event(event)?;
            }
            
            // Update state
            self.update().await?;
        }
        
        Ok(())
    }
    
    /// Updates application state.
    async fn update(&mut self) -> Result<()> {
        // Skip updates if paused
        if let Temporality::Live = self.state.temporality {
            // Update task states - convert to state::TaskUpdate with .into()
            if let Some(task_updates) = self.task_monitor.poll().await {
                // Convert monitor::task::TaskUpdate to state::TaskUpdate
                let state_update: crate::state::TaskUpdate = task_updates.into();
                self.state.update_tasks(vec![state_update]);
            }
            
            // Update backend states - convert to state::BackendUpdate with .into()
            if let Some(backend_updates) = self.backend_monitor.poll().await {
                // Convert monitor::backend::BackendUpdate to state::BackendUpdate
                let state_update: crate::state::BackendUpdate = backend_updates.into();
                self.state.update_backends(vec![state_update]);
            }
        }
        
        Ok(())
    }
    
    /// Handles input and other events.
    fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key) => self.handle_key_event(key),
            Event::Tick => Ok(()),
            Event::Resize(width, height) => {
                // Handle resize events
                self.ui.handle_resize(width, height);
                Ok(())
            }
        }
    }
    
    /// Handles keyboard input.
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        // Handle global keys first - similar to tokio-console's multi-level delegation
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                Ok(())
            },
            KeyCode::Char('p') => {
                self.toggle_pause();
                Ok(())
            },
            KeyCode::F(1) => {
                self.toggle_help();
                Ok(())
            },
            // Delegate to UI controller for view-specific handling
            _ => {
                // Process the UpdateKind result
                let update_kind = self.ui.handle_key_event(key, &mut self.state)?;
                
                // Process the update kind (if needed)
                match update_kind {
                    ui::UpdateKind::Quit => self.should_quit = true,
                    ui::UpdateKind::TogglePause => self.toggle_pause(),
                    ui::UpdateKind::ToggleHelp => self.toggle_help(),
                    _ => {} // Ignore other update kinds
                }
                
                Ok(())
            }
        }
    }
    
    /// Toggles pause state.
    fn toggle_pause(&mut self) {
        self.state.temporality = match self.state.temporality {
            Temporality::Live => Temporality::Paused,
            Temporality::Paused => Temporality::Live,
            Temporality::Pausing => Temporality::Live,
            Temporality::Unpausing => Temporality::Paused,
        };
    }
    
    /// Toggles help overlay.
    fn toggle_help(&mut self) {
        self.ui.toggle_help();
    }
}