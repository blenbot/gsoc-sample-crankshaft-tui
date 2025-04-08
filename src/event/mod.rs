//! Event handling for the Crankshaft TUI.
//!
//! This module manages terminal events (keyboard, resize, etc.) and provides
//! a structured way to handle them in the application. It uses the same event-driven
//! architecture pattern as tokio-console with a dedicated event loop.

pub mod handler;

pub use handler::{EventHandler, EventResult};

use std::time::Duration;
use eyre::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};

/// Default event polling interval.
pub const DEFAULT_TICK_RATE: Duration = Duration::from_millis(100);

/// Application events
#[derive(Debug, Clone)]
pub enum Event {
    /// Keyboard input event
    Key(KeyEvent),
    /// Terminal resize event
    Resize(u16, u16),
    /// Regular tick event for animations
    Tick,
}

/// Event dispatcher that collects terminal events.
pub struct EventDispatcher {
    /// Polling interval
    tick_rate: Duration,
}

impl EventDispatcher {
    /// Create a new event dispatcher with the default tick rate.
    pub fn new() -> Self {
        Self {
            tick_rate: DEFAULT_TICK_RATE,
        }
    }
    
    /// Set a custom tick rate.
    pub fn with_tick_rate(mut self, tick_rate: Duration) -> Self {
        self.tick_rate = tick_rate;
        self
    }
    
    /// Wait for and return the next event.
    pub fn next(&self) -> Result<Event> {
        if event::poll(self.tick_rate)? {
            match event::read()? {
                CrosstermEvent::Key(key) => Ok(Event::Key(key)),
                CrosstermEvent::Resize(width, height) => Ok(Event::Resize(width, height)),
                _ => Ok(Event::Tick),
            }
        } else {
            Ok(Event::Tick)
        }
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}