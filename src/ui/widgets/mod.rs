//! Custom widgets for the Crankshaft TUI.
//!
//! This module provides specialized widgets that implement efficient rendering patterns
//! inspired by tokio-console:
//! - Progressive sizing and layout based on available space
//! - Memory-efficient rendering with minimal allocations
//! - Context-aware styling and formatting
//! - Component-based design with clear separation of concerns

pub mod sparkline;
pub mod progress;
pub mod stat_panel;
pub mod tabbed_view;

pub use sparkline::Sparkline;
pub use progress::ProgressBar;
pub use stat_panel::StatPanel;
pub use tabbed_view::TabbedView;