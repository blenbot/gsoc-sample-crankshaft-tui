//! Task detail view showing comprehensive information for a specific task.



use ratatui::Frame;
use ratatui::layout::{Layout, Constraint, Direction, Rect};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs, List, ListState, ListItem};
use crossterm::event::KeyEvent;


use crate::ui::widgets::sparkline::Sparkline as CustomSparkline;
use crate::state::{AppState, TaskState, TaskStatus, ResourceSample};
use crate::ui::Theme;

/// Tab selection for task detail view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailTab {
    Info,
    Logs,
    Resources,
}

/// Task detail view showing comprehensive information for a specific task.
#[derive(Debug, Clone, PartialEq)]
pub struct TaskDetailView {
    /// ID of the task being viewed
    task_id: u64,
    /// Currently selected tab
    current_tab: DetailTab,
    /// Scroll position in logs view
    log_scroll: u16,
    /// List state for resource samples
    resource_list_state: ListState,
}

impl TaskDetailView {
    /// Create a new task detail view for the given task.
    pub fn new(task_id: u64) -> Self {
        Self {
            task_id,
            current_tab: DetailTab::Info,
            log_scroll: 0,
            resource_list_state: ListState::default(),
        }
    }
    
    /// Render the task detail view.
    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        app_state: &AppState,
        theme: &Theme,
    ) {
        // Find the task in the app state
        let task = match app_state.tasks.get(&self.task_id) {
            Some(task) => task,
            None => {
                // Task not found, show error message
                let error_text = Text::styled(
                    format!("Task with ID {} not found", self.task_id),
                    theme.error_style,
                );
                
                let error_widget = Paragraph::new(error_text)
                    .block(Block::default()
                        .title("Error")
                        .borders(Borders::ALL)
                        .style(theme.block_style));
                        
                frame.render_widget(error_widget, area);
                return;
            }
        };
        
        // Create a layout with sections for:
        // - Header with task summary
        // - Tabs
        // - Content area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Length(3),  // Tabs
                Constraint::Min(3),     // Content
            ])
            .split(area);
            
        // Render header with task summary
        self.render_header(frame, chunks[0], task, app_state, theme);
        
        // Render tabs
        self.render_tabs(frame, chunks[1], theme);
        
        // Render content based on selected tab
        match self.current_tab {
            DetailTab::Info => self.render_info_tab(frame, chunks[2], task, app_state, theme),
            DetailTab::Logs => self.render_logs_tab(frame, chunks[2], task, app_state, theme),
            DetailTab::Resources => self.render_resources_tab(frame, chunks[2], task, app_state, theme),
        }
    }
    
    /// Render the header with task summary.
    fn render_header(
        &self,
        frame: &mut Frame,
        area: Rect,
        task: &TaskState,
        _app_state: &AppState,
        theme: &Theme,
    ) {
        // Determine status style
        let status_style = match task.status {
            TaskStatus::Created => theme.created_style,
            TaskStatus::Queued => theme.queued_style,
            TaskStatus::Running => theme.running_style,
            TaskStatus::Completed => theme.completed_style,
            TaskStatus::Failed => theme.failed_style,
            TaskStatus::Cancelled => theme.cancelled_style,
        };
        
        // Create header text with task ID, name, and status
        let header_text = vec![
            Line::from(vec![
                Span::styled("Task ID: ", theme.label_style),
                Span::styled(task.id.to_string(), theme.value_style),
                Span::raw(" | "),
                Span::styled("Status: ", theme.label_style),
                Span::styled(task.status.to_string(), status_style),
            ]),
            Line::from(vec![
                Span::styled("Name: ", theme.label_style),
                Span::styled(&task.name, theme.value_style),
            ]),
        ];
        
        let header = Paragraph::new(header_text)
            .style(theme.normal_text)
            .block(Block::default().borders(Borders::ALL));
            
        frame.render_widget(header, area);
    }
    
    /// Render the tabs.
    fn render_tabs(
        &self,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
    ) {
        let tab_titles = ["Info", "Logs", "Resources"];
        
        let selected_tab = match self.current_tab {
            DetailTab::Info => 0,
            DetailTab::Logs => 1,
            DetailTab::Resources => 2,
        };
        
        let tabs = Tabs::new(tab_titles.into_iter().map(Line::from).collect::<Vec<_>>())
            .block(Block::default().borders(Borders::ALL))
            .style(theme.normal_text)
            .highlight_style(theme.selected_style)
            .select(selected_tab);
            
        frame.render_widget(tabs, area);
    }
    
    /// Render the info tab.
    fn render_info_tab(
        &self,
        frame: &mut Frame,
        area: Rect,
        task: &TaskState,
        _app_state: &AppState,
        theme: &Theme,
    ) {
        // Create a block for the content
        let block = Block::default()
            .title("Task Information")
            .borders(Borders::ALL)
            .style(theme.block_style);
            
        let _inner = block.inner(area);
        
        // Format task information
        let duration = if let Some(end_time) = task.end_time {
            format_duration(&(end_time - task.start_time))
        } else {
            format_duration(&(chrono::Utc::now() - task.start_time))
        };
        
        let progress = if let Some(progress) = task.progress {
            format!("{:.1}%", progress * 100.0)
        } else {
            "N/A".to_string()
        };
        
        let info_text = vec![
            Line::from(vec![
                Span::styled("Backend: ", theme.label_style),
                Span::styled(&task.backend, theme.value_style),
            ]),
            Line::from(vec![
                Span::styled("Start Time: ", theme.label_style),
                Span::styled(task.start_time.to_rfc3339(), theme.value_style),
            ]),
            Line::from(vec![
                Span::styled("End Time: ", theme.label_style),
                Span::styled(match task.end_time {
                    Some(time) => time.to_rfc3339(),
                    None => "Running".to_string(),
                }, theme.value_style),
            ]),
            Line::from(vec![
                Span::styled("Duration: ", theme.label_style),
                Span::styled(duration, theme.value_style),
            ]),
            Line::from(vec![
                Span::styled("Progress: ", theme.label_style),
                Span::styled(progress, theme.value_style),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Current Resource Usage:", theme.label_style),
            ]),
            Line::from(vec![
                Span::styled("  CPU: ", theme.label_style),
                Span::styled(format!("{:.1}%", task.cpu_usage), theme.value_style),
            ]),
            Line::from(vec![
                Span::styled("  Memory: ", theme.label_style),
                Span::styled(format!("{:.1} MB", task.memory_usage), theme.value_style),
            ]),
        ];
        
        let info = Paragraph::new(info_text)
            .style(theme.normal_text)
            .block(block);
            
        frame.render_widget(info, area);
    }
    
    /// Render the logs tab.
    fn render_logs_tab(
        &self,
        frame: &mut Frame,
        area: Rect,
        _task: &TaskState,
        app_state: &AppState,
        theme: &Theme,
    ) {
        // Create a block for the content
        let block = Block::default()
            .title("Task Logs")
            .borders(Borders::ALL)
            .style(theme.block_style);
            
        let _inner = block.inner(area);
        
        // Get log content from task details if available
        let logs = if let Some(details) = &app_state.current_task_details {
            details.borrow().logs.clone()
        } else {
            // No logs available
            vec!["No logs available for this task.".to_string()]
        };
        
        // Format log lines
        let log_content: Vec<Line> = logs.into_iter()
            .map(|line| Line::from(line))
            .collect();
        
        let logs_paragraph = Paragraph::new(log_content)
            .style(theme.normal_text)
            .block(block)
            .scroll((self.log_scroll, 0));
            
        frame.render_widget(logs_paragraph, area);
    }
    
    /// Render the resources tab.
    fn render_resources_tab(
        &self,
        frame: &mut Frame,
        area: Rect,
        task: &TaskState,
        app_state: &AppState,
        theme: &Theme,
    ) {
        // Split the area into resource graphs (left) and details (right)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),  // Graphs
                Constraint::Percentage(50),  // Details
            ])
            .split(area);
            
        // Get resource history if available
        let resource_samples = if let Some(details) = &app_state.current_task_details {
            details.borrow().resource_history.clone()
        } else {
            Vec::new()
        };
        
        // Render graphs
        self.render_resource_graphs(frame, chunks[0], task, resource_samples.as_slice(), theme);
        
        // Render resource details
        self.render_resource_details(frame, chunks[1], task, resource_samples.as_slice(), theme);
    }
    
    /// Render resource utilization graphs.
    fn render_resource_graphs(
        &self,
        frame: &mut Frame,
        area: Rect,
        task: &TaskState,
        samples: &[ResourceSample],
        theme: &Theme,
    ) {
        // Split area for CPU and memory graphs
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),  // CPU
                Constraint::Percentage(50),  // Memory
            ])
            .split(area);
            
        // Create CPU data
        let cpu_data: Vec<f64> = if samples.is_empty() {
            vec![task.cpu_usage as f64]
        } else {
            samples.iter().map(|s| s.cpu as f64).collect()
        };
        
        // Create memory data
        let memory_data: Vec<f64> = if samples.is_empty() {
            vec![task.memory_usage as f64]
        } else {
            samples.iter().map(|s| s.memory as f64).collect()
        };
        
        // Create CPU sparkline
        let cpu_block = Block::default()
            .title("CPU Usage")
            .borders(Borders::ALL)
            .style(theme.block_style);
        
        // Fixed: Use Sparkline::new instead of default() + data()
        let cpu_sparkline = CustomSparkline::new(&cpu_data)
            .block(cpu_block)
            .style(theme.sparkline_style)
            .max(100.0);
            
        // Create memory sparkline
        let memory_block = Block::default()
            .title("Memory Usage")
            .borders(Borders::ALL)
            .style(theme.block_style);
          
        let memory_sparkline = CustomSparkline::new(&memory_data)
            .block(memory_block)
            .style(theme.sparkline_style)
            .max(*memory_data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&100.0) * 1.1);
            
        frame.render_widget(cpu_sparkline, chunks[0]);
        frame.render_widget(memory_sparkline, chunks[1]);
    }
    
    /// Render resource details.
    fn render_resource_details(
        &self,
        frame: &mut Frame,
        area: Rect,
        _task: &TaskState,
        samples: &[ResourceSample],
        theme: &Theme,
    ) {
        // Create a block for the content
        let block = Block::default()
            .title("Resource Samples")
            .borders(Borders::ALL)
            .style(theme.block_style);
            
        // If no samples available, show message
        if samples.is_empty() {
            let message = Paragraph::new("No resource samples available")
                .style(theme.normal_text)
                .block(block);
                
            frame.render_widget(message, area);
            return;
        }
        
        // Format sample data as list items
        let items: Vec<ListItem> = samples.iter()
            .map(|sample| {
                let timestamp = sample.timestamp.to_rfc3339();
                let content = Line::from(vec![
                    Span::styled(format!("{}: ", timestamp), theme.label_style),
                    Span::styled(format!("CPU {:.1}%, ", sample.cpu), theme.value_style),
                    Span::styled(format!("Mem {:.1}MB", sample.memory), theme.value_style),
                ]);
                
                ListItem::new(content)
            })
            .collect();
            
        // Create and render the list
        let mut list_state = self.resource_list_state.clone();
        
        let list = List::new(items)
            .block(block)
            .style(theme.normal_text)
            .highlight_style(theme.selected_style);
            
        frame.render_stateful_widget(list, area, &mut list_state);
    }
    
    /// Handle keyboard input.
    pub fn handle_key_event(&mut self, key: KeyEvent, _app_state: &mut AppState) -> eyre::Result<()> {
        use crossterm::event::KeyCode;
        
        match key.code {
            // Tab navigation
            KeyCode::Tab | KeyCode::Right => self.next_tab(),
            KeyCode::BackTab | KeyCode::Left => self.prev_tab(),
            
            // Tab-specific handling
            _ => match self.current_tab {
                DetailTab::Info => { /* No special handling */ }
                
                DetailTab::Logs => match key.code {
                    KeyCode::Up | KeyCode::Char('k') => self.scroll_logs_up(),
                    KeyCode::Down | KeyCode::Char('j') => self.scroll_logs_down(),
                    KeyCode::Home | KeyCode::Char('g') => self.scroll_logs_top(),
                    KeyCode::End | KeyCode::Char('G') => self.scroll_logs_bottom(),
                    _ => {}
                },
                
                DetailTab::Resources => match key.code {
                    KeyCode::Up | KeyCode::Char('k') => self.select_prev_resource(),
                    KeyCode::Down | KeyCode::Char('j') => self.select_next_resource(),
                    _ => {}
                },
            }
        }
        
        Ok(())
    }
    
    /// Select the next tab.
    fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            DetailTab::Info => DetailTab::Logs,
            DetailTab::Logs => DetailTab::Resources,
            DetailTab::Resources => DetailTab::Info,
        };
    }
    
    /// Select the previous tab.
    fn prev_tab(&mut self) {
        self.current_tab = match self.current_tab {
            DetailTab::Info => DetailTab::Resources,
            DetailTab::Logs => DetailTab::Info,
            DetailTab::Resources => DetailTab::Logs,
        };
    }
    
    /// Scroll logs up.
    fn scroll_logs_up(&mut self) {
        self.log_scroll = self.log_scroll.saturating_sub(1);
    }
    
    /// Scroll logs down.
    fn scroll_logs_down(&mut self) {
        // In a real app, you'd check against the actual log size
        self.log_scroll = self.log_scroll.saturating_add(1);
    }
    
    /// Scroll logs to top.
    fn scroll_logs_top(&mut self) {
        self.log_scroll = 0;
    }
    
    /// Scroll logs to bottom.
    fn scroll_logs_bottom(&mut self) {
        // In a real app, you'd set this to (log_lines - visible_lines)
        // For now, just use a large number as placeholder
        self.log_scroll = 1000;
    }
    
    /// Select previous resource sample.
    fn select_prev_resource(&mut self) {
        let i = match self.resource_list_state.selected() {
            Some(i) => {
                if i > 0 {
                    i - 1
                } else {
                    0
                }
            }
            None => 0,
        };
        self.resource_list_state.select(Some(i));
    }
    
    /// Select next resource sample.
    fn select_next_resource(&mut self) {
        // In a real app, you'd check against the actual number of samples
        let i = match self.resource_list_state.selected() {
            Some(i) => i + 1,
            None => 0,
        };
        self.resource_list_state.select(Some(i));
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