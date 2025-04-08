//! Task list view showing all tasks with filtering and sorting.

use ratatui::Frame;
use ratatui::layout::{Layout, Constraint, Direction, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, TableState, Table, Row, Cell, Paragraph};
use ratatui::style::{Style, Color};

use crate::state::{AppState, TaskState, TaskStatus};
use crate::ui::Theme;

/// Sort fields for the task list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Id,
    Name,
    Status,
    Progress,
    Backend,
    Duration,
    CpuUsage,
    MemoryUsage,
}

/// Task list view showing all tasks with filtering and sorting.
pub struct TaskListView {
    /// What field to sort by
    sort_field: SortField,
    /// Sort in ascending order
    sort_ascending: bool,
    /// Table state for cursor position
    table_state: TableState,
}

impl Default for TaskListView {
    fn default() -> Self {
        Self {
            sort_field: SortField::Id,
            sort_ascending: true,
            table_state: TableState::default(),
        }
    }
}

impl TaskListView {
    /// Create a new task list view.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the sort field and direction.
    pub fn sort_by(mut self, field: SortField, ascending: bool) -> Self {
        self.sort_field = field;
        self.sort_ascending = ascending;
        self
    }
    
    /// Render the task list view.
    pub fn render(frame: &mut Frame, area: Rect, app_state: &AppState, theme: &Theme) {
        let mut view = Self::default();
        
        // If there's a selected task ID in the app state, select it in the table
        if let Some(task_id) = app_state.selected_task_id {
            // Find the index of the task in the sorted list
            let mut tasks: Vec<&TaskState> = app_state.tasks.values().collect();
            Self::sort_tasks(&mut tasks, view.sort_field, view.sort_ascending);
            
            if let Some(index) = tasks.iter().position(|task| task.id == task_id) {
                view.table_state.select(Some(index));
            }
        }
        
        // Create layout and render components
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),  // Header
                Constraint::Min(3),     // Task table
            ])
            .split(area);
            
        Self::render_header(frame, chunks[0], app_state, theme, &view);
        Self::render_tasks_table(frame, chunks[1], app_state, theme, &mut view);
    }
    
    /// Render the header with filter and search info.
    fn render_header(
        frame: &mut Frame,
        area: Rect,
        app_state: &AppState,
        theme: &Theme,
        view: &TaskListView,
    ) {
        let title = format!("Tasks ({} total)", app_state.tasks.len());
        
        // Show sort information
        let sort_info = format!(
            "Sort: {} {}",
            match view.sort_field {
                SortField::Id => "ID",
                SortField::Name => "Name",
                SortField::Status => "Status",
                SortField::Progress => "Progress",
                SortField::Backend => "Backend",
                SortField::Duration => "Duration",
                SortField::CpuUsage => "CPU",
                SortField::MemoryUsage => "Memory",
            },
            if view.sort_ascending { "↑" } else { "↓" }
        );
        
        let header_text = Line::from(vec![
            Span::styled(title, theme.header_style),
            Span::raw(" | "),
            Span::styled(sort_info, theme.label_style),
            Span::raw(" | "),
            Span::styled("Press Enter to view details", theme.help_style),
        ]);
        
        let header = Paragraph::new(header_text)
            .style(theme.normal_text)
            .block(Block::default().borders(Borders::BOTTOM));
            
        frame.render_widget(header, area);
    }
    
    /// Render the main task table.
    fn render_tasks_table(
        frame: &mut Frame,
        area: Rect,
        app_state: &AppState,
        theme: &Theme,
        view: &mut TaskListView,
    ) {
        // Create the table block
        let table_block = Block::default()
            .borders(Borders::ALL)
            .style(theme.block_style);
        
        // Create the table header
        let header_cells = ["ID", "Name", "Status", "Progress", "Duration", "Backend", "CPU", "Memory"]
            .iter()
            .map(|h| {
                Cell::from(*h).style(theme.header_style)
            });
        let header = Row::new(header_cells).style(theme.header_style);
        
        // Sort the tasks based on the current sort field and direction
        let mut tasks: Vec<&TaskState> = app_state.tasks.values().collect();
        Self::sort_tasks(&mut tasks, view.sort_field, view.sort_ascending);
        
        // Format task rows
        let rows = tasks.into_iter().map(|task| {
            format_task_row(task, view.table_state.selected() == Some(task.id.try_into().unwrap()))
        });
        
        // Create the table
        let table = Table::new(
            rows,
            [
                Constraint::Length(6),
                Constraint::Percentage(25),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(15),
                Constraint::Length(8),
                Constraint::Length(10),
            ]
        )
            .header(header)
            .block(table_block)
            .highlight_style(theme.selected_style);
        
        // Render the table with state
        frame.render_stateful_widget(table, area, &mut view.table_state);
    }
    
    /// Sort tasks by the given field.
    fn sort_tasks(tasks: &mut [&TaskState], field: SortField, ascending: bool) {
        tasks.sort_by(|a, b| {
            let cmp = match field {
                SortField::Id => a.id.cmp(&b.id),
                SortField::Name => a.name.cmp(&b.name),
                SortField::Status => a.status.to_string().cmp(&b.status.to_string()),
                SortField::Progress => a.progress.unwrap_or(0.0).partial_cmp(&b.progress.unwrap_or(0.0)).unwrap(),
                SortField::Backend => a.backend.cmp(&b.backend),
                SortField::Duration => {
                    let a_dur = if let Some(end) = a.end_time {
                        end - a.start_time
                    } else {
                        chrono::Utc::now() - a.start_time
                    };
                    
                    let b_dur = if let Some(end) = b.end_time {
                        end - b.start_time
                    } else {
                        chrono::Utc::now() - b.start_time
                    };
                    
                    a_dur.cmp(&b_dur)
                },
                SortField::CpuUsage => a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap(),
                SortField::MemoryUsage => a.memory_usage.partial_cmp(&b.memory_usage).unwrap(),
            };
            
            if ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });
    }
    
    /// Handle keyboard input.
    pub fn handle_key_event(&mut self, key: crossterm::event::KeyEvent, _app_state: &mut AppState) -> eyre::Result<()> {
        use crossterm::event::KeyCode;
        
        match key.code {
            // Navigation
            KeyCode::Down | KeyCode::Char('j') => self.next(),
            KeyCode::Up | KeyCode::Char('k') => self.previous(),
            KeyCode::Home | KeyCode::Char('g') => self.first(),
            KeyCode::End | KeyCode::Char('G') => self.last(),
            
            // Sorting
            KeyCode::Char('1') => self.toggle_sort(SortField::Id),
            KeyCode::Char('2') => self.toggle_sort(SortField::Name),
            KeyCode::Char('3') => self.toggle_sort(SortField::Status),
            KeyCode::Char('4') => self.toggle_sort(SortField::Progress),
            KeyCode::Char('5') => self.toggle_sort(SortField::Duration),
            KeyCode::Char('6') => self.toggle_sort(SortField::Backend),
            KeyCode::Char('7') => self.toggle_sort(SortField::CpuUsage),
            KeyCode::Char('8') => self.toggle_sort(SortField::MemoryUsage),
            
            // Toggle direction
            KeyCode::Char('i') => self.sort_ascending = !self.sort_ascending,
            
            _ => {}
        }
        
        Ok(())
    }
    
    /// Select the next task.
    pub fn next(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                i+1
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }
    
    /// Select the previous task.
    pub fn previous(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i > 0 {
                    i - 1
                } else {
                    0
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }
    
    /// Select the first task.
    pub fn first(&mut self) {
        self.table_state.select(Some(0));
    }
    
    /// Select the last task.
    pub fn last(&mut self) {
        // This requires knowing the number of tasks
        // In practice, you'd get this from the AppState
        // Just set a placeholder here
        self.table_state.select(Some(0));
    }
    
    /// Toggle sort by the given field.
    pub fn toggle_sort(&mut self, field: SortField) {
        if self.sort_field == field {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_field = field;
            self.sort_ascending = true;
        }
    }
}

/// Format a duration as a human-readable string.
fn format_duration(duration: &chrono::Duration) -> String {
    let seconds = duration.num_seconds();
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    }
}

fn format_task_row(task: &TaskState, is_selected: bool) -> Row {
    let progress_display = if let Some(progress) = task.progress {
        let percentage = (progress * 100.0).round() as u8;
        let bar_width = 20;
        let filled = (bar_width as f32 * progress) as usize;
        let empty = bar_width - filled;
        
        format!("[{}{}] {}%", 
            "█".repeat(filled), 
            "░".repeat(empty), 
            percentage
        )
    } else {
        match task.status {
            TaskStatus::Created => "[    pending    ]".to_string(),
            TaskStatus::Queued => "[    waiting    ]".to_string(),
            TaskStatus::Running => "[    running    ]".to_string(),
            TaskStatus::Completed => "[   completed   ]".to_string(),
            TaskStatus::Failed => "[     failed     ]".to_string(),
            TaskStatus::Cancelled => "[   cancelled   ]".to_string(),
        }
    };
    
    // Create the row with the progress bar
    Row::new(vec![
        Cell::from(format!("{}", task.id)).style(Style::default()),
        Cell::from(task.name.clone()),
        Cell::from(task.status.to_string()).style(get_status_style(task.status)),
        Cell::from(progress_display),
        Cell::from(format_duration(&task.elapsed())),
    ])
}

fn get_status_style(status: TaskStatus) -> Style {
    match status {
        TaskStatus::Created => Style::default().fg(Color::Blue),
        TaskStatus::Queued => Style::default().fg(Color::Yellow),
        TaskStatus::Running => Style::default().fg(Color::Green),
        TaskStatus::Completed => Style::default().fg(Color::Cyan),
        TaskStatus::Failed => Style::default().fg(Color::Red),
        TaskStatus::Cancelled => Style::default().fg(Color::Gray),
    }
}





