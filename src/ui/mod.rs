//! UI components for the Crankshaft TUI.
//! 
//! This module contains the user interface components for the Crankshaft TUI.
//! It's organized around a multi-level state machine pattern similar to tokio-console,
//! with different views that can be switched between, and each view having its own state.

mod dashboard;
pub mod task_list;
pub mod task_detail;
pub mod backend_view;
pub mod log_view;
pub mod theme;
pub mod help;
pub mod widgets;

pub use dashboard::DashboardView;
pub use task_list::TaskListView;
pub use task_detail::TaskDetailView;
pub use backend_view::BackendView;
pub use log_view::LogView;
pub use theme::Theme;
pub use help::HelpView;

use crossterm::event::KeyEvent;
use eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::state::{AppState, Temporality};

/// The result of updating the UI in response to user input.
pub enum UpdateKind {
    /// Quit the application
    Quit,
    /// Toggle help overlay
    ToggleHelp,
    /// Toggle pause state
    TogglePause,  // Add this variant
    /// Select a task
    SelectTask(u64),
    /// Exit task detail view
    ExitTaskView,
    /// Select a backend
    SelectBackend(String),
    /// Exit backend detail view
    ExitBackendView,
    /// Other update (no action needed)
    Other,
}

/// Available views in the application.
#[derive(Debug, Clone, PartialEq)]
pub enum ViewState {
    /// Dashboard overview of all tasks and backends
    Dashboard,
    /// List of all tasks
    TasksList,
    /// List of all backends
    BackendsList,
    /// Detailed view of a specific task
    TaskInstance(TaskDetailView),
    /// Detailed view of a specific backend
    BackendInstance(BackendView),
}

/// Main UI controller.
pub struct Ui {
    /// Current view state
    state: ViewState,
    /// Whether to show help overlay
    show_help: bool,
    /// UI theme
    theme: Theme,
    /// Terminal width
    terminal_width: u16,
    /// Terminal height
    terminal_height: u16,
    /// Current animation frame (for spinners, progress bars, etc)
    animation_frame: usize,
}

impl Ui {
    /// Create a new UI controller.
    pub fn new() -> Self {
        Self {
            state: ViewState::Dashboard,
            show_help: false,
            theme: Theme::default(),
            terminal_width: 80,  
            terminal_height: 24,
            animation_frame: 0,
        }
    }
    
    /// Get the current view state.
    pub fn current_view(&self) -> &ViewState {
        &self.state
    }
    
    /// Toggle help overlay.
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
    
    /// Set the theme.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }
    
    /// Handle keyboard input.
    pub fn handle_key_event(&mut self, key: KeyEvent, app_state: &mut AppState) -> Result<UpdateKind> {
        use crossterm::event::KeyCode;
        
        // Global shortcuts first
        match key.code {
            KeyCode::F(1) | KeyCode::Char('?') => return Ok(UpdateKind::ToggleHelp),
            KeyCode::Char('q') => return Ok(UpdateKind::Quit),
            KeyCode::Char('d') => {
                self.state = ViewState::Dashboard;
                return Ok(UpdateKind::Other);
            },
            KeyCode::Char('t') => {
                self.state = ViewState::TasksList;
                return Ok(UpdateKind::Other);
            },
            KeyCode::Char('b') => {
                self.state = ViewState::BackendsList;
                return Ok(UpdateKind::Other);
            },
            KeyCode::Char('p') => return Ok(UpdateKind::TogglePause),
            _ => {} 
        }
        
        // Delegate to view-specific handlers
        match &mut self.state {
            ViewState::Dashboard => {
                self.handle_dashboard_input(key, app_state)
            },
            ViewState::TasksList => {
                self.handle_tasks_list_input(key, app_state)
            },
            ViewState::BackendsList => {
                self.handle_backends_list_input(key, app_state)
            },
            ViewState::TaskInstance(view) => {
                // Create mutable view for the handler
                let mut view_clone = view.clone();
                let result = self.handle_task_detail_input(&mut view_clone, key, app_state);
                // If the view was modified, update it in self.state
                if let ViewState::TaskInstance(ref mut v) = self.state {
                    *v = view_clone;
                }
                result
            },
            ViewState::BackendInstance(view) => {
                // Create mutable view for the handler
                let mut view_clone = view.clone();
                let result = self.handle_backend_detail_input(&mut view_clone, key, app_state);
                // If the view was modified, update it in self.state
                if let ViewState::BackendInstance(ref mut v) = self.state {
                    *v = view_clone;
                }
                result
            },
        }
    }
    
    /// Render the UI.
    pub fn render(&self, frame: &mut Frame, app_state: &AppState) {
        let area = frame.size();
        
        // Render current view
        match &self.state {
            ViewState::Dashboard => self.render_dashboard(frame, area, app_state),
            ViewState::TasksList => self.render_tasks_list(frame, area, app_state),
            ViewState::BackendsList => self.render_backends_list(frame, area, app_state),
            ViewState::TaskInstance(view) => self.render_task_detail(view, frame, area, app_state),
            ViewState::BackendInstance(view) => self.render_backend_detail(view, frame, area, app_state),
        }
        
        // Render help overlay if active (always on top)
        if self.show_help {
            self.render_help(frame, area, app_state);
        }
        
        // Render status line with app state
        self.render_status_line(frame, area, app_state);
    }

    /// Render the UI in a specific area
    pub fn render_in_area(&self, frame: &mut Frame, app_state: &AppState, area: Rect) {
        match &self.state {
            ViewState::Dashboard => DashboardView::render(frame, area, app_state, &self.theme),
            ViewState::TasksList => TaskListView::render(frame, area, app_state, &self.theme),
            ViewState::BackendsList => BackendView::render_list(frame, area, app_state, &self.theme),
            ViewState::TaskInstance(view) => view.render(frame, area, app_state, &self.theme),
            ViewState::BackendInstance(view) => view.render(frame, area, app_state, &self.theme),
        }
        
        // Render help if active
        if self.show_help {
            self.render_help(frame, area, app_state);
        }
    }

    pub fn navigate_to(&mut self, view: ViewState) {
        self.state = view;
    }
    
    
    
    /// Handle terminal resize events
    pub fn handle_resize(&mut self, width: u16, height: u16) {
        // Store dimensions for layouts to use later
        self.terminal_width = width;
        self.terminal_height = height;
    }
    
    /// Update animation frames for UI elements
    pub fn update_animations(&mut self) {
        self.animation_frame = (self.animation_frame + 1) % 4;
    }
    
    // Private methods for input handling
    
    fn handle_dashboard_input(&mut self, _key: KeyEvent, _app_state: &mut AppState) -> Result<UpdateKind> {
        // Dashboard-specific input handling
        Ok(UpdateKind::Other)
    }
    
    fn handle_tasks_list_input(&mut self, key: KeyEvent, app_state: &mut AppState) -> Result<UpdateKind> {
        use crossterm::event::KeyCode;
        
        match key.code {
            KeyCode::Enter => {
                // Find selected task and switch to detail view
                if let Some(task_id) = app_state.selected_task_id() {
                    // Create a copy of the task_id before moving it into the new view
                    let task_id_value = *task_id;
                    self.state = ViewState::TaskInstance(TaskDetailView::new(task_id_value));
                    return Ok(UpdateKind::SelectTask(task_id_value));
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app_state.select_next_task();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app_state.select_prev_task();
            }
            _ => {}
        }
        
        Ok(UpdateKind::Other)
    }
    
    fn handle_backends_list_input(&mut self, key: KeyEvent, app_state: &mut AppState) -> Result<UpdateKind> {
        use crossterm::event::KeyCode;
        
        match key.code {
            KeyCode::Enter => {
                // Find selected backend and switch to detail view
                if let Some(backend_name) = app_state.selected_backend_name() {
                    self.state = ViewState::BackendInstance(BackendView::new(backend_name.clone()));
                    return Ok(UpdateKind::SelectBackend(backend_name));
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app_state.select_next_backend();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app_state.select_prev_backend();
            }
            _ => {}
        }
        
        Ok(UpdateKind::Other)
    }
    
    fn handle_task_detail_input(&mut self, view: &mut TaskDetailView, key: KeyEvent, app_state: &mut AppState) -> Result<UpdateKind> {
        use crossterm::event::KeyCode;
        
        match key.code {
            KeyCode::Esc => {
                self.state = ViewState::TasksList;
                return Ok(UpdateKind::ExitTaskView);
            }
            _ => {
                // Pass input to task detail view
                view.handle_key_event(key, app_state)?;
            }
        }
        
        Ok(UpdateKind::Other)
    }
    
    fn handle_backend_detail_input(&mut self, view: &mut BackendView, key: KeyEvent, app_state: &mut AppState) -> Result<UpdateKind> {
        use crossterm::event::KeyCode;
        
        match key.code {
            KeyCode::Esc => {
                self.state = ViewState::BackendsList;
                return Ok(UpdateKind::ExitBackendView);
            }
            _ => {
                // Pass input to backend detail view
                view.handle_key_event(key, app_state)?;
            }
        }
        
        Ok(UpdateKind::Other)
    }
    
    // Private methods for rendering
    
    fn render_dashboard(&self, frame: &mut Frame, area: Rect, app_state: &AppState) {
        DashboardView::render(frame, area, app_state, &self.theme);
    }
    
    fn render_tasks_list(&self, frame: &mut Frame, area: Rect, app_state: &AppState) {
        TaskListView::render(frame, area, app_state, &self.theme);
    }
    
    fn render_backends_list(&self, frame: &mut Frame, area: Rect, app_state: &AppState) {
        BackendView::render_list(frame, area, app_state, &self.theme);
    }
    
    fn render_task_detail(&self, view: &TaskDetailView, frame: &mut Frame, area: Rect, app_state: &AppState) {
        view.render(frame, area, app_state, &self.theme);
    }
    
    fn render_backend_detail(&self, view: &BackendView, frame: &mut Frame, area: Rect, app_state: &AppState) {
        view.render(frame, area, app_state, &self.theme);
    }
    
    fn render_help(&self, frame: &mut Frame, area: Rect, app_state: &AppState) {
        HelpView::render(frame, area, app_state, &self.theme, &self.state);
    }
    
    fn render_status_line(&self, frame: &mut Frame, area: Rect, app_state: &AppState) {
        // Create a status line at the bottom of the screen
        let status_area = Rect::new(0, area.height - 1, area.width, 1);
        
        // Format status based on application state
        let status = match app_state.temporality {
            Temporality::Live => "LIVE",
            Temporality::Paused => "PAUSED",
            Temporality::Pausing => "PAUSING...",
            Temporality::Unpausing => "UNPAUSING...",
        };
        
        // Count active tasks and backends
        let active_tasks = app_state.active_task_count();
        let total_tasks = app_state.tasks.len();
        let backends = app_state.backends.len();
        
        // Format the status line
        let status_text = format!(
            "{} | Tasks: {}/{} | Backends: {} | Press ? for help", 
            status, active_tasks, total_tasks, backends
        );
        
        let status_style = match app_state.temporality {
            Temporality::Live => self.theme.status_live,
            Temporality::Paused => self.theme.status_paused,
            Temporality::Pausing => self.theme.status_paused,
            Temporality::Unpausing => self.theme.status_live,
        };
        
        let status_widget = ratatui::widgets::Paragraph::new(status_text)
            .style(status_style)
            .block(ratatui::widgets::Block::default());
            
        frame.render_widget(status_widget, status_area);
    }
}