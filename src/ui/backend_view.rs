//! Backend view showing information about execution backends.
//!
//! This view demonstrates multiple key patterns from tokio-console:
//! 1. Progressive disclosure (overview to details)
//! 2. Context-aware rendering with conditional components
//! 3. Tab-based navigation for complex data
//! 4. Cross-entity navigation (backend -> tasks)

use ratatui::Frame;
use ratatui::layout::{Layout, Constraint, Direction, Rect};
use ratatui::style::{Color, Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs, Table, Row, Cell, TableState, Gauge, BarChart};
use crossterm::event::{KeyEvent, KeyCode};
use eyre::Result;
use rand::Rng;

use crate::state::{AppState, BackendState, HealthStatus, BackendKind, TaskStatus};
use crate::ui::Theme;
use crate::ui::widgets::sparkline::Sparkline;

/// Tab selection for backend detail view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendTab {
    Info,
    Tasks,
    Resources,
}

impl BackendTab {
    /// Get the next tab in the cycle.
    pub fn next(&self) -> Self {
        match self {
            Self::Info => Self::Tasks,
            Self::Tasks => Self::Resources,
            Self::Resources => Self::Info,
        }
    }

    /// Get the previous tab in the cycle.
    pub fn prev(&self) -> Self {
        match self {
            Self::Info => Self::Resources,
            Self::Resources => Self::Info,
            Self::Tasks => Self::Resources,
        }
    }
}

/// Backend view for displaying backend information.
#[derive(Debug, Clone, PartialEq)] 
pub struct BackendView {
    /// Name of the backend being viewed
    backend_name: String,
    /// Currently selected tab
    current_tab: BackendTab,
    /// Table state for task list
    task_table_state: TableState,
    /// Resource history time window (in minutes)
    resource_time_window: u16,
}

impl BackendView {
    /// Create a new backend view for the given backend.
    pub fn new(backend_name: String) -> Self {
        Self {
            backend_name,
            current_tab: BackendTab::Info,
            task_table_state: TableState::default(),
            resource_time_window: 10,
        }
    }
    
    /// Handle key events for this view.
    pub fn handle_key_event(&mut self, key: KeyEvent, app_state: &mut AppState) -> Result<()> {
        match key.code {
            // Tab navigation
            KeyCode::Tab | KeyCode::Right => self.current_tab = self.current_tab.next(),
            KeyCode::BackTab | KeyCode::Left => self.current_tab = self.current_tab.prev(),
            
            // Task list navigation (when on Tasks tab)
            KeyCode::Down | KeyCode::Char('j') if self.current_tab == BackendTab::Tasks => {
                if let Some(_backend) = app_state.backends.get(&self.backend_name) {
                    let task_count = app_state.tasks
                        .values()
                        .filter(|t| t.backend == self.backend_name)
                        .count();
                    
                    if task_count > 0 {
                        let new_index = match self.task_table_state.selected() {
                            Some(i) => (i + 1) % task_count,
                            None => 0,
                        };
                        self.task_table_state.select(Some(new_index));
                    }
                }
            }
            KeyCode::Up | KeyCode::Char('k') if self.current_tab == BackendTab::Tasks => {
                if let Some(_backend) = app_state.backends.get(&self.backend_name) {
                    let task_count = app_state.tasks
                        .values()
                        .filter(|t| t.backend == self.backend_name)
                        .count();
                    
                    if task_count > 0 {
                        let new_index = match self.task_table_state.selected() {
                            Some(i) => {
                                if i == 0 { task_count - 1 } else { i - 1 }
                            }
                            None => 0,
                        };
                        self.task_table_state.select(Some(new_index));
                    }
                }
            }
            
            // Resource time window adjustment
            KeyCode::Char('+') if self.current_tab == BackendTab::Resources => {
                self.resource_time_window = self.resource_time_window.saturating_add(5);
            }
            KeyCode::Char('-') if self.current_tab == BackendTab::Resources => {
                self.resource_time_window = self.resource_time_window.saturating_sub(5).max(1);
            }
            
            _ => {}
        }
        
        Ok(())
    }
    
    /// Render a list of all backends.
    pub fn render_list(
        frame: &mut Frame,
        area: Rect,
        app_state: &AppState,
        theme: &Theme,
    ) {
        let block = Block::default()
            .title("Backends")
            .borders(Borders::ALL)
            .style(theme.block_style);
            
        // Create a table for backends
        let header = ["Name", "Type", "Tasks", "Status", "CPU", "Memory"]
            .iter()
            .map(|h| Cell::from(*h).style(theme.header_style));
            
        let header = Row::new(header)
            .style(theme.header_style);
        
        // Create rows for each backend - utilizing tokio-console's adaptive rendering
        let rows = app_state.backends.values().map(|backend| {
            let status_style = match backend.health {
                HealthStatus::Healthy => theme.healthy_style,
                HealthStatus::Degraded => theme.warning_style,
                HealthStatus::Unhealthy => theme.error_style,
                HealthStatus::Unknown => theme.normal_text,
            };
            
            // Create a row with cells
            Row::new([
                Cell::from(backend.name.clone()),
                Cell::from(format!("{:?}", backend.kind)),
                Cell::from(format!("{}/{}", backend.running_tasks, backend.total_tasks)),
                Cell::from(backend.health.to_string()).style(status_style),
                Cell::from(format!("{:.1}%", backend.cpu_usage)),
                Cell::from(format!("{:.1}%", backend.memory_usage)),
            ])
        });
        
        // Create and render the table
        let mut table_state = TableState::default();
        
        // Find the currently selected backend, if any
        if let Some((index, _)) = app_state.backends.values()
            .enumerate()
            .find(|(_, b)| app_state.selected_backend_name().map_or(false, |selected| selected == b.name))
        {
            table_state.select(Some(index));
        }
        
        let table = Table::new(
            rows,
            [
                Constraint::Percentage(25),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
            ]
        )
            .header(header)
            .block(block)
            .highlight_style(theme.selected_style)
            .highlight_symbol(">> ");
            
        frame.render_stateful_widget(table, area, &mut table_state);
    }
    
    /// Render the backend detail view.
    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        app_state: &AppState,
        theme: &Theme,
    ) {
        // Split the area into a tabs area and a content area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);
            
        // Create tab titles
        let titles = [
            Span::styled("Info", if self.current_tab == BackendTab::Info { theme.selected_style } else { theme.normal_text }),
            Span::styled("Tasks", if self.current_tab == BackendTab::Tasks { theme.selected_style } else { theme.normal_text }),
            Span::styled("Resources", if self.current_tab == BackendTab::Resources { theme.selected_style } else { theme.normal_text }),
        ];
        let tabs = Tabs::new(titles.to_vec())
            .block(Block::default().borders(Borders::ALL).title(format!("Backend: {}", self.backend_name)))
            .highlight_style(theme.selected_style)
            .select(self.current_tab as usize);
            
        frame.render_widget(tabs, chunks[0]);
        
        // Render the content based on the selected tab
        match self.current_tab {
            BackendTab::Info => self.render_info_tab(frame, chunks[1], app_state, theme),
            BackendTab::Tasks => self.render_tasks_tab(frame, chunks[1], app_state, theme),
            BackendTab::Resources => self.render_resources_tab(frame, chunks[1], app_state, theme),
        }
    }
    
    /// Render the Info tab.
    fn render_info_tab(
        &self,
        frame: &mut Frame,
        area: Rect,
        app_state: &AppState,
        theme: &Theme,
    ) {
        if let Some(backend) = app_state.backends.get(&self.backend_name) {
            // Split area for different info sections
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(6),
                    Constraint::Length(8),
                    Constraint::Min(0),
                ])
                .margin(1)
                .split(area);
                
            // Basic backend info
            let info_text = vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(&backend.name),
                ]),
                Line::from(vec![
                    Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{:?}", backend.kind)),
                ]),
                Line::from(vec![
                    Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(
                        backend.health.to_string(),
                        match backend.health {
                            HealthStatus::Healthy => theme.healthy_style,
                            HealthStatus::Degraded => theme.warning_style,
                            HealthStatus::Unhealthy => theme.error_style,
                            HealthStatus::Unknown => theme.normal_text,
                        }
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Tasks: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{} running / {} total", backend.running_tasks, backend.total_tasks)),
                ]),
            ];
            
            let info_widget = Paragraph::new(info_text)
                .block(Block::default().borders(Borders::ALL).title("Backend Information"))
                .style(theme.normal_text);
                
            frame.render_widget(info_widget, chunks[0]);
            
            // Resource utilization
            let utilization_title = match backend.kind {
                BackendKind::Docker => "Container Resources",
                BackendKind::TES => "Cloud Resources",
                BackendKind::Generic => "Resources",
                BackendKind::Unknown => "Resources",
                BackendKind::Local => "Local Resources", 
            };
            
            frame.render_widget(
                Block::default().borders(Borders::ALL).title(utilization_title),
                chunks[1]
            );
            
            // CPU usage gauge
            let cpu_gauge = Gauge::default()
                .block(Block::default().title("CPU Usage"))
                .gauge_style(
                    Style::default().fg(
                        if backend.cpu_usage > 80.0 {
                            Color::Red
                        } else if backend.cpu_usage > 50.0 {
                            Color::Yellow
                        } else {
                            Color::Green
                        }
                    )
                )
                .percent(backend.cpu_usage as u16);
                
            // Memory usage gauge
            let memory_gauge = Gauge::default()
                .block(Block::default().title("Memory Usage"))
                .gauge_style(
                    Style::default().fg(
                        if backend.memory_usage > 80.0 {
                            Color::Red
                        } else if backend.memory_usage > 50.0 {
                            Color::Yellow
                        } else {
                            Color::Green
                        }
                    )
                )
                .percent(backend.memory_usage as u16);
                
            // Layout for resource gauges
            let resource_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                ])
                .margin(1)
                .split(chunks[1]);
                
            frame.render_widget(cpu_gauge, resource_chunks[0]);
            frame.render_widget(memory_gauge, resource_chunks[1]);
            
            // Backend-specific configuration info
            let config_text = match backend.kind {
                BackendKind::Docker => vec![
                    Line::from(vec![
                        Span::styled("Image Handling: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw("Native container image support"),
                    ]),
                    Line::from(vec![
                        Span::styled("Limits: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw("Enforced through cgroups"),
                    ]),
                ],
                BackendKind::TES => vec![
                    Line::from(vec![
                        Span::styled("Endpoint: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw("Cloud task execution service"),
                    ]),
                    Line::from(vec![
                        Span::styled("Features: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw("Batch processing, autoscaling"),
                    ]),
                ],
                BackendKind::Generic => vec![
                    Line::from(vec![
                        Span::styled("Executor: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw("Shell commands"),
                    ]),
                    Line::from(vec![
                        Span::styled("Transport: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw("Local or SSH"),
                    ]),
                ],
                BackendKind::Unknown => vec![
                    Line::from("Unknown backend type")
                ],
                BackendKind::Local => vec![  // Add this match arm
                    Line::from(vec![
                        Span::styled("Runtime: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw("Local machine execution"),
                    ]),
                    Line::from(vec![
                        Span::styled("Features: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw("Direct file access, native performance"),
                    ]),
                ],
            };
            
            let config_widget = Paragraph::new(config_text)
                .block(Block::default().borders(Borders::ALL).title("Configuration"))
                .style(theme.normal_text);
                
            frame.render_widget(config_widget, chunks[2]);
        } else {
            // Backend not found
            let text = vec![
                Line::from(vec![
                    Span::styled("Error: ", theme.error_style),
                    Span::raw(format!("Backend '{}' not found", self.backend_name)),
                ]),
            ];
            
            let widget = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("Backend Not Found"))
                .style(theme.normal_text);
                
            frame.render_widget(widget, area);
        }
    }
    
    /// Render the Tasks tab showing tasks running on this backend.
    fn render_tasks_tab(
        &self,
        frame: &mut Frame,
        area: Rect,
        app_state: &AppState,
        theme: &Theme,
    ) {
        // Collect tasks for this backend
        let tasks: Vec<_> = app_state.tasks
            .values()
            .filter(|t| t.backend == self.backend_name)
            .collect();
            
        if tasks.is_empty() {
            let text = vec![
                Line::from("No tasks found for this backend"),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Run tasks with: "),
                    Span::styled("crankshaft run task.json", Style::default().add_modifier(Modifier::BOLD)),
                ]),
            ];
            
            let widget = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("No Tasks"))
                .style(theme.normal_text);
                
            frame.render_widget(widget, area);
            return;
        }
        
        // Create a table for tasks
        let header = ["ID", "Name", "Status", "Progress", "CPU", "Memory", "Duration"]
            .iter()
            .map(|h| Cell::from(*h).style(theme.header_style));
            
        let header = Row::new(header).style(theme.header_style);
        
        // Create task rows
        let rows = tasks.iter().map(|task| {
            let status_style = match task.status {
                TaskStatus::Created => Style::default().fg(Color::Blue),
                TaskStatus::Queued => Style::default().fg(Color::Yellow),
                TaskStatus::Running => Style::default().fg(Color::Green),
                TaskStatus::Completed => Style::default().fg(Color::Cyan),
                TaskStatus::Failed => Style::default().fg(Color::Red),
                TaskStatus::Cancelled => Style::default().fg(Color::Gray),
            };
            
            // Create progress bar
            let progress = task.progress.unwrap_or(0.0);
            let progress_percent = (progress * 100.0) as u16;
            let bar_width = 10;
            let filled = ((bar_width as f32) * progress) as usize;
            let empty = bar_width - filled;
            let progress_bar = format!("{}{} {}%",
                "█".repeat(filled),
                "░".repeat(empty),
                progress_percent
            );
            
            // Format duration
            let duration = if let Some(end_time) = task.end_time {
                let duration = end_time - task.start_time;
                format!("{}s", duration.num_seconds())
            } else {
                let now = chrono::Utc::now();
                let duration = now - task.start_time;
                format!("{}s", duration.num_seconds())
            };
            
            Row::new([
                Cell::from(task.id.to_string()),
                Cell::from(task.name.clone()),
                Cell::from(task.status.to_string()).style(status_style),
                Cell::from(progress_bar),
                Cell::from(format!("{:.1}%", task.cpu_usage)),
                Cell::from(format!("{:.1}MB", task.memory_usage)),
                Cell::from(duration),
            ])
        });
        
        // Create and render table
        let table = Table::new(
            rows,
            [
                Constraint::Percentage(10),
                Constraint::Percentage(25),
                Constraint::Percentage(10),
                Constraint::Percentage(15),
                Constraint::Percentage(10),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
            ]
        )
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Tasks on this Backend"))
            .highlight_style(theme.selected_style)
            .highlight_symbol(">> ");
            
        // Clone task state since we need to pass a mutable reference
        let mut task_table_state = self.task_table_state.clone();
        frame.render_stateful_widget(table, area, &mut task_table_state);
    }
    
    /// Render the Resources tab showing resource utilization over time.
    fn render_resources_tab(
        &self,
        frame: &mut Frame,
        area: Rect,
        app_state: &AppState,
        theme: &Theme,
    ) {
        if let Some(backend) = app_state.backends.get(&self.backend_name) {
            // Split area for different resource visualizations
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(8), // CPU
                    Constraint::Length(8), // Memory
                    Constraint::Min(0),    // Task count
                ])
                .margin(1)
                .split(area);
                
            // Generate synthetic resource history for the demo
            // In a real implementation, this would come from the backend
            let cpu_history = generate_resource_history(30, backend.cpu_usage);
            let memory_history = generate_resource_history(30, backend.memory_usage);
            
            // Convert to f64 for sparkline
            let cpu_data: Vec<f64> = cpu_history.iter().map(|x| *x as f64).collect();
            let memory_data: Vec<f64> = memory_history.iter().map(|x| *x as f64).collect();
            
            // CPU usage sparkline
            let cpu_sparkline = Sparkline::new(&cpu_data)
                .block(Block::default().borders(Borders::ALL).title("CPU Usage (%)"))
                .style(Style::default().fg(Color::Green))
                .max(100.0); // Scale to 100%
                
            frame.render_widget(cpu_sparkline, chunks[0]);
            
            // Memory usage sparkline
            let memory_sparkline = Sparkline::new(&memory_data)
                .block(Block::default().borders(Borders::ALL).title("Memory Usage (%)"))
                .style(Style::default().fg(Color::Blue))
                .max(100.0); // Scale to 100%
                
            frame.render_widget(memory_sparkline, chunks[1]);
            
            // Task count history
            let task_history_block = Block::default()
                .borders(Borders::ALL)
                .title(format!("Task History (last {} minutes)", self.resource_time_window));
                
            // First define the chart_area before rendering to it
            let chart_area = chunks[2];
            frame.render_widget(task_history_block.clone(), chart_area);
            
            // Use inner area of the block for the charts
            let chart_area = task_history_block.inner(chart_area);
            
            // Create synthetic task history data
            let task_data = generate_task_history(backend);
            let _labels = ["Running", "Completed", "Failed"];
            
            
            let running_data = [
                ("5m ago", task_data[0][0]),
                ("4m ago", task_data[0][1]),
                ("3m ago", task_data[0][2]),
                ("2m ago", task_data[0][3]),
                ("1m ago", task_data[0][4]),
                ("now", task_data[0][5]),
            ];

            let completed_data = [
                ("5m ago", task_data[1][0]),
                ("4m ago", task_data[1][1]),
                ("3m ago", task_data[1][2]),
                ("2m ago", task_data[1][3]),
                ("1m ago", task_data[1][4]),
                ("now", task_data[1][5]),
            ];

            let failed_data = [
                ("5m ago", task_data[2][0]),
                ("4m ago", task_data[2][1]),
                ("3m ago", task_data[2][2]),
                ("2m ago", task_data[2][3]),
                ("1m ago", task_data[2][4]),
                ("now", task_data[2][5]),
            ];

            // Create three charts and split the chart area into thirds vertically
            let chart_sub_areas = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                ])
                .split(chart_area);

            // Render three separate bar charts with different colors
            let running_chart = BarChart::default()
                .data(&running_data)
                .bar_width(9)
                .bar_style(Style::default().fg(Color::Green))
                .value_style(Style::default().add_modifier(Modifier::BOLD))
                .label_style(Style::default().fg(Color::White))
                .bar_gap(2)
                .block(Block::default().title("Running Tasks"));

            let completed_chart = BarChart::default()
                .data(&completed_data)
                .bar_width(9)
                .bar_style(Style::default().fg(Color::Cyan))
                .value_style(Style::default().add_modifier(Modifier::BOLD))
                .label_style(Style::default().fg(Color::White))
                .bar_gap(2)
                .block(Block::default().title("Completed Tasks"));

            let failed_chart = BarChart::default()
                .data(&failed_data)
                .bar_width(9)
                .bar_style(Style::default().fg(Color::Red))
                .value_style(Style::default().add_modifier(Modifier::BOLD))
                .label_style(Style::default().fg(Color::White))
                .bar_gap(2)
                .block(Block::default().title("Failed Tasks"));

            frame.render_widget(running_chart, chart_sub_areas[0]);
            frame.render_widget(completed_chart, chart_sub_areas[1]);
            frame.render_widget(failed_chart, chart_sub_areas[2]);
            
            // Help text for adjusting time window
            let help_text = vec![
                Line::from(vec![
                    Span::styled("+ ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw("Increase time window  "),
                    Span::styled("- ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw("Decrease time window"),
                ]),
            ];
            
            let help_widget = Paragraph::new(help_text)
                .style(theme.help_style);
                
            let help_area = Rect {
                x: area.x + 2,
                y: area.height.saturating_sub(2) + area.y,
                width: area.width.saturating_sub(4),
                height: 1,
            };
                
            frame.render_widget(help_widget, help_area);
        } else {
            // Backend not found
            let text = vec![
                Line::from(vec![
                    Span::styled("Error: ", theme.error_style),
                    Span::raw(format!("Backend '{}' not found", self.backend_name)),
                ]),
            ];
            
            let widget = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("Backend Not Found"))
                .style(theme.normal_text);
                
            frame.render_widget(widget, area);
        }
    }
}

// Helper function to generate synthetic resource history for demo
fn generate_resource_history(points: usize, current_value: f32) -> Vec<f32> {
    let mut history = Vec::with_capacity(points);
    let mut value = current_value;
    
    // Work backwards from current value
    for _ in 0..points {
        history.push(value);
        // Random walk with regression to mean
        let change = rand::thread_rng().gen::<f32>() * 10.0 - 5.0;
        value = (value + change).clamp(0.0, 100.0);
    }
    
    history.reverse();
    history
}

// Helper function to generate synthetic task history for demo
fn generate_task_history(backend: &BackendState) -> [[u64; 6]; 3] {
    let running = backend.running_tasks as u64;
    let total = backend.total_tasks as u64;
    let completed = (total - running) / 2;
    let failed = total - running - completed;
    
    // Generate 6 time points of data
    [
        // Running tasks over time
        [
            running.saturating_sub(2),
            running.saturating_sub(1),
            running,
            running,
            running.saturating_add(1),
            running,
        ],
        // Completed tasks over time
        [
            completed.saturating_sub(3),
            completed.saturating_sub(2),
            completed.saturating_sub(1),
            completed,
            completed,
            completed.saturating_add(1),
        ],
        // Failed tasks over time
        [
            failed,
            failed,
            failed,
            failed.saturating_add(1),
            failed.saturating_add(1),
            failed.saturating_add(1),
        ],
    ]
}

