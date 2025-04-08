//! Dashboard view for the Crankshaft TUI.
//!
//! This module implements the main dashboard view, which shows:
//! 1. Task summary with counts per status
//! 2. Backend summary with health status
//! 3. Resource usage across all backends
//! 4. Recent events
//!
//! Key design patterns demonstrated:
//! - Adaptive layout based on terminal size (from tokio-console)
//! - Progressive disclosure of information
//! - Context-aware rendering
//! - Bidirectional entity relationships


use std::collections::HashMap;
use ratatui::Frame;
use ratatui::layout::{Layout, Constraint, Direction, Rect};
use ratatui::style::{Color, Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Table, Row, Cell, 
                       TableState, BarChart, List, ListItem, Wrap};

use crate::state::{AppState, TaskStatus, HealthStatus, Temporality};
use crate::ui::Theme;
use crate::ui::widgets::sparkline::Sparkline as CustomSparkline;

/// Dashboard view showing an overview of all tasks and backends.
pub struct DashboardView;

impl DashboardView {
    /// Render the dashboard view.
    pub fn render(
        frame: &mut Frame,  // Updated: removed <B> generic parameter
        area: Rect,
        app_state: &AppState,
        theme: &Theme,
    ) {
        // Determine the best layout based on terminal size (inspired by tokio-console's adaptive layout)
        let direction = if area.width > 100 { Direction::Horizontal } else { Direction::Vertical };
        let constraints = if direction == Direction::Horizontal {
            [Constraint::Percentage(50), Constraint::Percentage(50)]
        } else {
            [Constraint::Percentage(40), Constraint::Percentage(60)]
        };
        
        let chunks = Layout::default()
            .direction(direction)
            .constraints(constraints)
            .split(area);
            
        // Left/top section: Task summary and global resources
        Self::render_task_summary(frame, chunks[0], app_state, theme);
        
        // Right/bottom section: Backend summary and events
        Self::render_backend_summary(frame, chunks[1], app_state, theme);
    }
    
    /// Render task summary section.
    fn render_task_summary(
        frame: &mut Frame,
        area: Rect,
        app_state: &AppState,
        theme: &Theme,
    ) {
        // Divide the area into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),  // Task status summary
                Constraint::Length(10), // Resource usage
                Constraint::Min(0),     // Recent tasks
            ])
            .split(area);
            
        // Render the task status summary
        Self::render_task_status_summary(frame, chunks[0], app_state, theme);
        
        // Render the resource usage
        Self::render_resource_usage(frame, chunks[1], app_state, theme);
        
        // Render the recent tasks
        Self::render_recent_tasks(frame, chunks[2], app_state, theme);
    }
    
    /// Render backend summary section.
    fn render_backend_summary(
        frame: &mut Frame,
        area: Rect,
        app_state: &AppState,
        theme: &Theme,
    ) {
        // Divide the area into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60), // Backend table
                Constraint::Percentage(40), // Events
            ])
            .split(area);
            
        // Render the backend table
        Self::render_backend_table(frame, chunks[0], app_state, theme);
        
        // Render the events
        Self::render_events(frame, chunks[1], app_state, theme);
    }
    
    /// Render task status summary.
    fn render_task_status_summary(
        frame: &mut Frame,
        area: Rect,
        app_state: &AppState,
        theme: &Theme,
    ) {
        // Count tasks by status
        let mut status_counts = HashMap::new();
        status_counts.insert(TaskStatus::Created, 0);
        status_counts.insert(TaskStatus::Queued, 0);
        status_counts.insert(TaskStatus::Running, 0);
        status_counts.insert(TaskStatus::Completed, 0);
        status_counts.insert(TaskStatus::Failed, 0);
        status_counts.insert(TaskStatus::Cancelled, 0);
        
        for task in app_state.tasks.values() {
            *status_counts.entry(task.status).or_insert(0) += 1;
        }
        
        // Calculate total
        let total_tasks = app_state.tasks.len();
        
        // Create status summary text
        let mut text = vec![
            Line::from(vec![
                Span::styled("Total Tasks: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(total_tasks.to_string()),
            ]),
        ];
        
        // Add colored status counts
        let status_styles = [
            (TaskStatus::Created, Color::Blue),
            (TaskStatus::Queued, Color::Yellow),
            (TaskStatus::Running, Color::Green),
            (TaskStatus::Completed, Color::Cyan),
            (TaskStatus::Failed, Color::Red),
            (TaskStatus::Cancelled, Color::Gray),
        ];
        
        for (status, color) in status_styles.iter() {
            let count = status_counts.get(status).unwrap_or(&0);
            text.push(Line::from(vec![
                Span::styled(
                    format!("{}: ", status.to_string()),
                    Style::default().fg(*color).add_modifier(Modifier::BOLD)
                ),
                Span::raw(count.to_string()),
            ]));
        }
        
        // Create bars for visual representation
        let bars_data = [
            ("Running", status_counts.get(&TaskStatus::Running).unwrap_or(&0) * 100),
            ("Queued", status_counts.get(&TaskStatus::Queued).unwrap_or(&0) * 100),
            ("Failed", status_counts.get(&TaskStatus::Failed).unwrap_or(&0) * 100),
        ];
        
        // Render the paragraph and bar chart side by side
        let inner_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(60),
            ])
            .split(area.inner(&ratatui::layout::Margin { 
                vertical: 1, 
                horizontal: 2 
            }));
            
        // Render the text paragraph
        let paragraph = Paragraph::new(text)
            .style(theme.normal_text);
            
        frame.render_widget(paragraph, inner_chunks[0]);
        
        // Render the bar chart
        let bars = BarChart::default()
            .data(&bars_data)
            .bar_width(7)
            .bar_style(Style::default().fg(Color::Green))
            .value_style(Style::default().add_modifier(Modifier::BOLD))
            .label_style(Style::default().fg(Color::White));
            
        frame.render_widget(bars, inner_chunks[1]);
        
        // Render the block around everything
        frame.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .title("Task Summary")
                .style(theme.block_style),
            area,
        );
    }
    
    /// Render resource usage graphs.
    fn render_resource_usage(
        frame: &mut Frame,
        area: Rect,
        app_state: &AppState,
        theme: &Theme,
    ) {
        // Get resource data from state
        let cpu_data: Vec<f64> = app_state.resources.cpu_history
            .iter()
            .map(|p| p.value as f64)
            .collect();
            
        let memory_data: Vec<f64> = app_state.resources.memory_history
            .iter()
            .map(|p| p.value as f64)
            .collect();
            
        // Create inner layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .margin(1)
            .split(area.inner(&ratatui::layout::Margin { 
                vertical: 0, 
                horizontal: 0 
            }));
            
        // Render CPU sparkline
        let cpu_sparkline = CustomSparkline::new(&cpu_data)
            .block(Block::default()
                .title("CPU Usage (%)")
                .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT))
            .style(Style::default().fg(Color::Green))
            .max(100.0);
            
        frame.render_widget(cpu_sparkline, chunks[0]);
        
        // Render Memory sparkline
        let memory_sparkline = CustomSparkline::new(&memory_data)
            .block(Block::default()
                .title("Memory Usage (%)")
                .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT))
            .style(Style::default().fg(Color::Blue))
            .max(100.0);
            
        frame.render_widget(memory_sparkline, chunks[1]);
        
        // Render the overall block
        frame.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .title("Resource Usage")
                .style(theme.block_style),
            area,
        );
    }
    
    /// Render recent tasks list.
    fn render_recent_tasks(
        frame: &mut Frame,
        area: Rect,
        app_state: &AppState,
        theme: &Theme,
    ) {
        // Sort tasks by start time (most recent first)
        let mut recent_tasks: Vec<_> = app_state.tasks.values().collect();
        recent_tasks.sort_by(|a, b| b.start_time.cmp(&a.start_time));
        
        // Take only the 5 most recent tasks
        let recent_tasks = recent_tasks.into_iter().take(5);
        
        // Create list items
        let items: Vec<ListItem> = recent_tasks
            .map(|task| {
                // Format the task item with status color
                let status_style = match task.status {
                    TaskStatus::Created => Style::default().fg(Color::Blue),
                    TaskStatus::Queued => Style::default().fg(Color::Yellow),
                    TaskStatus::Running => Style::default().fg(Color::Green),
                    TaskStatus::Completed => Style::default().fg(Color::Cyan),
                    TaskStatus::Failed => Style::default().fg(Color::Red),
                    TaskStatus::Cancelled => Style::default().fg(Color::Gray),
                };
                
                // Create a formatted line for the task
                let line = Line::from(vec![
                    Span::styled(format!("[{}] ", task.status.to_string()), status_style),
                    Span::styled(task.name.clone(), Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!(" (ID: {})", task.id)),
                ]);
                
                ListItem::new(line)
            })
            .collect();
            
        // Create the list widget
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Recent Tasks"))
            .style(theme.normal_text)
            .highlight_style(theme.selected_style)
            .highlight_symbol(">> ");
            
        frame.render_widget(list, area);
    }
    
    /// Render backend table.
    fn render_backend_table(
        frame: &mut Frame,
        area: Rect,
        app_state: &AppState,
        theme: &Theme,
    ) {
        // Create header row
        let header = ["Name", "Type", "Tasks", "Status", "Utilization"]
            .iter()
            .map(|h| Cell::from(*h).style(theme.header_style));
            
        let header = Row::new(header)
            .style(theme.header_style);
            
        // Create rows
        let rows = app_state.backends.values().map(|backend| {
            // Style based on health status
            let status_style = match backend.health {
                HealthStatus::Healthy => theme.healthy_style   ,
                HealthStatus::Degraded => theme.warning_style,
                HealthStatus::Unhealthy => theme.error_style,
                HealthStatus::Unknown => theme.normal_text,
            };
            
            // Create utilization bar
            let utilization = backend.utilization() * 100.0;
            let bar_width = 10;
            let filled = (bar_width as f32 * backend.utilization()) as usize;
            let empty = bar_width - filled;
            let bar = format!("{}{} {:.1}%",
                "█".repeat(filled),
                "░".repeat(empty),
                utilization
            );
            
            Row::new([
                Cell::from(backend.name.clone()),
                Cell::from(format!("{:?}", backend.kind)),
                Cell::from(format!("{}/{}", backend.running_tasks, backend.total_tasks)),
                Cell::from(backend.health.to_string()).style(status_style),
                Cell::from(bar),
            ])
        });
        
        // Create the table
        let table = Table::new(
            rows,
            &[
                Constraint::Percentage(20),  // Name
                Constraint::Percentage(15),  // Type
                Constraint::Percentage(15),  // Tasks
                Constraint::Percentage(15),  // Status
                Constraint::Percentage(35),  // Utilization
            ]
        )
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Backends"))
            .highlight_style(theme.selected_style)
            .highlight_symbol(">> ");
            
        // Render the table with stateful highlighting
        let mut state = TableState::default();
        
        // Find the currently selected backend, if any
        if let Some((index, _)) = app_state.backends.values()
            .enumerate()
            .find(|(_, b)| app_state.selected_backend_name().map_or(false, |selected| selected == b.name))
        {
            state.select(Some(index));
        }
        
        frame.render_stateful_widget(table, area, &mut state);
    }
    
    /// Render system events and notifications.
    fn render_events(
        frame: &mut Frame,
        area: Rect,
        app_state: &AppState,
        theme: &Theme,
    ) {
        // Create placeholder events
        // In a real implementation, these would come from the application state
        let events = vec![
            (chrono::Utc::now(), "Engine connected successfully", Color::Green),
            (chrono::Utc::now() - chrono::Duration::seconds(30), "Task 'genome-analysis' completed", Color::Cyan),
            (chrono::Utc::now() - chrono::Duration::seconds(45), "Docker backend reports healthy status", Color::Green),
            (chrono::Utc::now() - chrono::Duration::minutes(2), "TES backend reports degraded status", Color::Yellow),
            (chrono::Utc::now() - chrono::Duration::minutes(5), "Task 'data-processing' failed", Color::Red),
        ];
        
        // Format events as text
        let text: Vec<Line> = events.into_iter().map(|(time, message, color)| {
            Line::from(vec![
                Span::styled(
                    format!("[{}] ", time.format("%H:%M:%S")),
                    Style::default().add_modifier(Modifier::BOLD)
                ),
                Span::styled(message, Style::default().fg(color)),
            ])
        }).collect();
        
        // Create the paragraph
        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Events"))
            .style(theme.normal_text)
            .wrap(Wrap { trim: true });
            
        frame.render_widget(paragraph, area);
    }
    
    /// Render the system status indicator.
    pub fn render_status(
        frame: &mut Frame,
        area: Rect,
        app_state: &AppState,
        theme: &Theme,
    ) {
        // Determine the overall system status based on backends
        let (status_text, status_style) = if app_state.backends.values().any(|b| b.health == HealthStatus::Unhealthy) {
            ("SYSTEM ALERT", theme.error_style)
        } else if app_state.backends.values().any(|b| b.health == HealthStatus::Degraded) {
            ("DEGRADED", theme.warning_style)
        } else if app_state.backends.values().all(|b| b.health == HealthStatus::Healthy) {
            ("HEALTHY", theme.healthy_style)
        } else {
            ("UNKNOWN", theme.normal_text)
        };
        
        // Add temporality indicator
        let status_indicator = match app_state.temporality {
            Temporality::Live => "▶ LIVE",
            Temporality::Paused => "⏸ PAUSED",
            Temporality::Pausing => "⏸ PAUSING...",
            Temporality::Unpausing => "▶ RESUMING...",
        };
        
        // Format the complete status line
        let text = Line::from(vec![
            Span::styled(status_indicator, match app_state.temporality {
                Temporality::Live | Temporality::Unpausing => theme.healthy_style,
                Temporality::Paused | Temporality::Pausing => theme.warning_style,
            }),
            Span::raw(" | "),
            Span::styled(status_text, status_style),
            Span::raw(" | "),
            Span::raw(format!("Tasks: {}/{} active/total", 
                app_state.tasks.values().filter(|t| t.is_active()).count(),
                app_state.tasks.len()
            )),
            Span::raw(" | "),
            Span::raw(format!("Backends: {}", app_state.backends.len())),
            Span::raw(" | "),
            Span::styled("Press ? for help", theme.help_style),
        ]);
        
        // Create the paragraph
        let paragraph = Paragraph::new(text)
            .style(theme.normal_text);
            
        frame.render_widget(paragraph, area);
    }
}



