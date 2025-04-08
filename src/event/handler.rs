//! Event handling logic for the Crankshaft TUI.
//!
//! This module defines handlers for terminal events and
//! application state transitions as a result of those events.

use std::sync::{Arc, Mutex};
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use color_eyre::Result;
use futures::Stream;
use crate::event;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use std::time::Duration;
use super::Event;

use crate::state::{AppState, Temporality};
use crate::ui::{Ui, UpdateKind, ViewState, TaskDetailView, BackendView};

/// Result of event handling.
#[derive(Debug, Clone, PartialEq)]
pub enum EventResult {
    /// Event was handled, continue running
    Continue,
    /// Exit the application
    Exit,
    /// Event was ignored
    Ignored,
}

/// Event handler for processing terminal events.
pub struct EventHandler {
    /// Application state
    app_state: Arc<Mutex<AppState>>,
    /// UI controller
    ui: Ui,
    /// Add a channel receiver for events
    event_rx: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    /// Create a new event handler.
    pub fn new(app_state: Arc<Mutex<AppState>>, ui: Ui) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        
        // Spawn a task that polls for events and sends them to the channel
        let _handle = tokio::spawn(async move {
            let dispatcher = event::EventDispatcher::new();
            loop {
                match dispatcher.next() {
                    Ok(event) => {
                        if tx.send(event).is_err() {
                            break;
                        }
                    }
                    Err(_) => {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                }
            }
        });
        
        Self { app_state, ui, event_rx: rx }
    }
    
    /// Handle an event.
    pub fn handle(&mut self, event: Event) -> Result<EventResult> {
        match event {
            Event::Key(key) => self.handle_key_event(key),
            Event::Resize(width, height) => {
                // Instead of storing in state, just update the UI
                // Remove the terminal size fields from state
                self.ui.handle_resize(width, height);
                Ok(EventResult::Continue)
            },
            Event::Tick => self.handle_tick(),
        }
    }
    
    /// Handle a key event.
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<EventResult> {
        // Check for application-wide keyboard shortcuts first
        match key.code {
            // Control-C exits the application
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Ok(EventResult::Exit);
            },
            // Control-D also exits the application (Unix convention)
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Ok(EventResult::Exit);
            },
            _ => {}
        }
        
        // Get mutable access to app state
        let mut state = self.app_state.lock().unwrap();
        
        // Let the UI handle the key event
        let update_kind = match self.ui.handle_key_event(key, &mut state) {
            Ok(update) => update,
            Err(_) => return Ok(EventResult::Ignored),
        };
        
        // Process the update result
        match update_kind {
            UpdateKind::Quit => Ok(EventResult::Exit),
            
            UpdateKind::SelectTask(task_id) => {
                state.select_task(task_id);
                self.ui.navigate_to(ViewState::TaskInstance(TaskDetailView::new(task_id)));
                Ok(EventResult::Continue)
            },
            
            UpdateKind::ExitTaskView => {
                state.deselect_task();
                self.ui.navigate_to(ViewState::TasksList);
                Ok(EventResult::Continue)
            },
            
            UpdateKind::SelectBackend(backend_name) => {
                state.selected_backend = Some(backend_name.clone());
                self.ui.navigate_to(ViewState::BackendInstance(BackendView::new(backend_name)));
                Ok(EventResult::Continue)
            },
            
            UpdateKind::ExitBackendView => {
                state.selected_backend = None;
                self.ui.navigate_to(ViewState::BackendsList);
                Ok(EventResult::Continue)
            },
            
            UpdateKind::ToggleHelp => {
                self.ui.toggle_help();
                Ok(EventResult::Continue)
            },
            
            UpdateKind::TogglePause => {
                // Toggle between Live and Paused state
                state.temporality = match state.temporality {
                    Temporality::Live => Temporality::Paused,
                    Temporality::Paused => Temporality::Live,
                    Temporality::Pausing => Temporality::Live,
                    Temporality::Unpausing => Temporality::Paused,
                };
                Ok(EventResult::Continue)
            },
            
            UpdateKind::Other => Ok(EventResult::Continue),
        }
    }
    
    /// Handle a tick event.
    fn handle_tick(&mut self) -> Result<EventResult> {
        self.ui.update_animations();
        
        Ok(EventResult::Continue)
    }

    
    fn poll_event(&mut self, cx: &mut Context<'_>) -> Poll<Option<Event>> {
        Pin::new(&mut self.event_rx).poll_recv(cx)
    }
}

// Stream for EventHandler
impl Stream for EventHandler {
    type Item = Event;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.poll_event(cx)
    }
}