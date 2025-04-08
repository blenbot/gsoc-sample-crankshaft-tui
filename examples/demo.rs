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
    event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use ratatui::prelude::Rect;
use ratatui::widgets::Paragraph;
use ratatui::style::{Style, Color};
use std::time::Duration;
use crankshaft_tui::state::HealthStatus;
use tokio::time::sleep;

use crankshaft_tui::state::{AppState, TaskUpdate, BackendUpdate, Temporality};
use crankshaft_tui::ui::{Ui, UpdateKind, ViewState};
use crankshaft_tui::monitor::{TaskMonitor, BackendMonitor};

async fn display_startup_animation(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Box<dyn Error>> {
    // Define the ASCII art logo
    let logo = vec![
        r"  _____                _        _           __ _   ",
        r" / ____|              | |      | |         / _| |  ",
        r"| |     _ __ __ _ _ __| | _____| |__   __ _| |_| |_ ",
        r"| |    | '__/ _` | '_ \ |/ / __| '_ \ / _` |  _| __|",
        r"| |____| | | (_| | | | |   <\__ \ | | | (_| | | | |_",
        r" \_____|_|  \__,_|_| |_|_|\_\___/_| |_|\__,_|_|  \__|",
    ];

    // Calculate logo dimensions
    let logo_height = logo.len();
    let logo_width = logo.iter().map(|line| line.len()).max().unwrap_or(0);

    // Render the logo centered in the terminal
    terminal.draw(|frame| {
        let size = frame.size();
        let start_row = (size.height.saturating_sub(logo_height as u16)) / 2;
        let start_col = (size.width.saturating_sub(logo_width as u16)) / 2;

        for (row_idx, line) in logo.iter().enumerate() {
            let y = start_row + row_idx as u16;
            if y < size.height {
                let paragraph = Paragraph::new(line.clone()).style(Style::default().fg(Color::Cyan));
                frame.render_widget(paragraph, Rect::new(start_col, y, logo_width as u16, 1));
            }
        }
    })?;

    // Pause for 2 seconds to display the logo
    sleep(Duration::from_secs(2)).await;

    Ok(())
}

fn render_dashboard_header(frame: &mut ratatui::Frame, app_state: &AppState, ui: &Ui) {
    let header_height = 3;
    let header_area = Rect::new(0, 0, frame.size().width, header_height);
    
    // Get system info
    let cpu_usage = app_state.resources.cpu_current;
    let mem_usage = app_state.resources.memory_current;
    
    // Create stylish header
    let title = " 🔧 CRANKSHAFT MONITORING DASHBOARD ";
    let subtitle = format!(" CPU: {:.1}% | Memory: {:.1}% | Mode: {} ", 
        cpu_usage, 
        mem_usage,
        match app_state.temporality {
            Temporality::Live => "LIVE",
            Temporality::Paused => "PAUSED",
            _ => "TRANSITION",
        }
    );
    
    // Create header block
    let header = ratatui::widgets::Paragraph::new(ratatui::text::Text::from(vec![
        ratatui::text::Line::from(vec![
            ratatui::text::Span::styled(
                title, 
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(ratatui::style::Modifier::BOLD)
            ),
        ]),
        ratatui::text::Line::from(vec![
            ratatui::text::Span::styled(
                subtitle,
                Style::default().fg(Color::White)
            ),
        ]),
    ]))
    .block(ratatui::widgets::Block::default()
        .borders(ratatui::widgets::Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .style(Style::default().bg(Color::Black)));
    
    frame.render_widget(header, header_area);
}

fn draw_help_modal(frame: &mut ratatui::Frame) {
    let area = frame.size();
    let popup_area = centered_rect(60, 70, area);
    
    // Create a clear background with shadow effect
    let shadow_area = Rect::new(
        popup_area.x + 1,
        popup_area.y + 1,
        popup_area.width,
        popup_area.height
    );
    frame.render_widget(ratatui::widgets::Clear, shadow_area);
    frame.render_widget(
        Paragraph::new("").style(Style::default().bg(Color::DarkGray)),
        shadow_area
    );
    
    // Clear the main help area
    frame.render_widget(ratatui::widgets::Clear, popup_area);
    
    // Help content
    let help_text = vec![
        ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("Crankshaft TUI Keyboard Shortcuts", 
                Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD))
        ]),
        ratatui::text::Line::from(""),
        ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("Navigation", 
                Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::UNDERLINED))
        ]),
        ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("d", Style::default().fg(Color::Green)),
            ratatui::text::Span::raw(" - Dashboard view")
        ]),
        ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("t", Style::default().fg(Color::Green)),
            ratatui::text::Span::raw(" - Tasks list view")
        ]),
        ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("b", Style::default().fg(Color::Green)),
            ratatui::text::Span::raw(" - Backends list view")
        ]),
        ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("Tab", Style::default().fg(Color::Green)),
            ratatui::text::Span::raw(" - Cycle through views")
        ]),
        ratatui::text::Line::from(""),
        ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("Task & Backend Selection", 
                Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::UNDERLINED))
        ]),
        ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("↑/↓", Style::default().fg(Color::Green)),
            ratatui::text::Span::raw(" - Navigate list items")
        ]),
        ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("Enter", Style::default().fg(Color::Green)),
            ratatui::text::Span::raw(" - Select item / Show details")
        ]),
        ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("Esc", Style::default().fg(Color::Green)),
            ratatui::text::Span::raw(" - Return to list view")
        ]),
        ratatui::text::Line::from(""),
        ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("General", 
                Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::UNDERLINED))
        ]),
        ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("p", Style::default().fg(Color::Green)),
            ratatui::text::Span::raw(" - Toggle pause/live updates")
        ]),
        ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("?", Style::default().fg(Color::Green)),
            ratatui::text::Span::raw(" - Toggle this help screen")
        ]),
        ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("q", Style::default().fg(Color::Green)),
            ratatui::text::Span::raw(" - Quit application")
        ]),
    ];
    
    // Enhanced rendering with background and borders
    let help_widget = ratatui::widgets::Paragraph::new(help_text)
        .block(ratatui::widgets::Block::default()
            .title(" Help ")
            .title_style(Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD))
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .style(Style::default().bg(Color::Black))
        .wrap(ratatui::widgets::Wrap { trim: true });
    
    frame.render_widget(help_widget, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_width = (r.width * percent_x) / 100;
    let popup_height = (r.height * percent_y) / 100;
    
    let popup_x = (r.width - popup_width) / 2;
    let popup_y = (r.height - popup_height) / 2;
    
    Rect::new(popup_x, popup_y, popup_width, popup_height)
}

fn render_status_bar(
    frame: &mut ratatui::Frame,
    area: Rect,
    state: &AppState,
    ui: &Ui,
    start_time: std::time::Instant,
    last_key: &str,
) {
    // Calculate uptime
    let uptime = start_time.elapsed();
    let uptime_str = format!(
        "{}:{:02}:{:02}", 
        uptime.as_secs() / 3600,
        (uptime.as_secs() % 3600) / 60, 
        uptime.as_secs() % 60
    );
    
    // Get task statistics
    let active_tasks = state.active_task_count();
    let total_tasks = state.tasks.len();
    
    // Count backends by health status
    let healthy_backends = state.backends.values().filter(|b| b.health == HealthStatus::Healthy).count();
    let total_backends = state.backends.len();
    
    // Build status text
    let status_text = format!(
        " {} | View: {:?} | Tasks: {}/{} | Backends: {}/{} | Uptime: {} | [?]Help | [q]Quit",
        match state.temporality {
            Temporality::Live => "🟢 LIVE",
            Temporality::Paused => "⏸️ PAUSED",
            _ => "⟳ UPDATING",
        },
        ui.current_view(),
        active_tasks,
        total_tasks,
        healthy_backends, 
        total_backends,
        uptime_str
    );
    
    frame.render_widget(
        Paragraph::new(status_text)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White)),
        area
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    
    // Show the startup animation first
    display_startup_animation(&mut terminal).await?;

    // Set up application state with mutex for thread safety
    let app_state = Arc::new(Mutex::new(AppState::new()));
    
    // Create a UI instance
    let mut ui = Ui::new();
    
    // Set up monitors
    let mut task_monitor = TaskMonitor::new();
    let mut backend_monitor = BackendMonitor::new();
    
    // Initialize monitors with simulated data
    task_monitor.connect("demo://localhost").await?;
    backend_monitor.connect("demo://localhost").await?;
    
    let mut last_key = String::new();
    let mut current_view = ViewState::Dashboard;
    let mut show_help = false;
    
    // Add this at the beginning of main():
    let start_time = std::time::Instant::now();
    
    // Run the event loop
    let mut should_exit = false;
    while !should_exit {
        // Replace your terminal.draw call with this:
        terminal.draw(|frame| {
            if let Ok(state) = app_state.lock() {
                // Create the main content area (leaving space for header and status)
                let total_area = frame.size();
                let header_height = 3;
                let status_height = 1;
                
                // Define areas for different UI sections
                let header_area = Rect::new(0, 0, total_area.width, header_height);
                let status_area = Rect::new(0, total_area.height - status_height, total_area.width, status_height);
                let content_area = Rect::new(
                    0, 
                    header_height, 
                    total_area.width, 
                    total_area.height - header_height - status_height
                );
                
                // Render custom header
                render_dashboard_header(frame, &state, &ui);
                
                // Use UI's render_in_area method to render content in the proper area
                ui.render_in_area(frame, &state, content_area);
                
                // Render dynamic status bar
                render_status_bar(frame, status_area, &state, &ui, start_time, &last_key);
                
                // Draw help modal if enabled
                if show_help {
                    draw_help_modal(frame);
                }
            }
        })?;
        
        // Handle events with a short timeout
        if let Ok(poll_result) = event::poll(Duration::from_millis(50)) {
            if poll_result {
                // An event is available, read it
                if let Ok(event) = event::read() {
                    match event {
                        Event::Key(key) => {
                            if key.kind == KeyEventKind::Press {
                                last_key = format!("{:?}", key);
                                
                                // Global shortcuts
                                if key.code == KeyCode::Char('q') {
                                    should_exit = true;
                                    continue;
                                }

                                // Toggle help screen
                                if key.code == KeyCode::Char('?') || key.code == KeyCode::F(1) {
                                    show_help = !show_help;
                                    continue;
                                }
                                
                                // Process key events through the UI controller
                                if let Ok(mut state) = app_state.lock() {
                                    let result = ui.handle_key_event(key, &mut state)?;
                                    
                                    // Update current_view after processing key (so it stays in sync)
                                    current_view = ui.current_view().clone();
                                    
                                    // Process the UpdateKind result
                                    match result {
                                        UpdateKind::Quit => should_exit = true,
                                        UpdateKind::TogglePause => {
                                            // Toggle state
                                            state.temporality = match state.temporality {
                                                Temporality::Live => Temporality::Paused,
                                                Temporality::Paused => Temporality::Live,
                                                Temporality::Pausing => Temporality::Live,
                                                Temporality::Unpausing => Temporality::Paused,
                                            };
                                        },
                                        UpdateKind::SelectTask(task_id) => {
                                            // Task was selected, details already handled by UI
                                            println!("Selected task: {}", task_id);
                                        },
                                        UpdateKind::SelectBackend(backend_name) => {
                                            // Backend was selected, details already handled by UI
                                            println!("Selected backend: {}", backend_name);
                                        },
                                        _ => {}
                                    }
                                }
                            }
                        },
                        Event::Resize(width, height) => {
                            ui.handle_resize(width, height);
                        },
                        _ => {}
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