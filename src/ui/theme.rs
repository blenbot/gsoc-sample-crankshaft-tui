//! UI theme definition.

use ratatui::style::{Color, Modifier, Style};

/// Theme for the application UI.
#[derive(Debug, Clone)]
pub struct Theme {
    // Basic styles
    pub normal_text: Style,
    pub selected_style: Style,
    pub block_style: Style,
    pub header_style: Style,
    pub label_style: Style,
    pub value_style: Style,
    
    // Status styles
    pub error_style: Style,
    pub help_style: Style,
    pub status_live: Style,
    pub status_paused: Style,
    
    // Key styles
    pub key_style: Style,
    
    // Task status styles
    pub created_style: Style,
    pub queued_style: Style,
    pub running_style: Style,
    pub completed_style: Style,
    pub failed_style: Style,
    pub cancelled_style: Style,
    
    // Backend status styles
    pub healthy_style: Style,
    pub warning_style: Style,
    pub critical_style: Style,
    pub offline_style: Style,
    
    // Chart and graph styles
    pub sparkline_style: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            // Basic styles
            normal_text: Style::default().fg(Color::White),
            selected_style: Style::default().fg(Color::Black).bg(Color::White),
            block_style: Style::default(),
            header_style: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            label_style: Style::default().fg(Color::Gray),
            value_style: Style::default().fg(Color::White),
            
            // Status styles
            error_style: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            help_style: Style::default().fg(Color::Gray),
            status_live: Style::default().fg(Color::Green),
            status_paused: Style::default().fg(Color::Yellow),
            
            // Key styles
            key_style: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            
            // Task status styles
            created_style: Style::default().fg(Color::Blue),
            queued_style: Style::default().fg(Color::Cyan),
            running_style: Style::default().fg(Color::Yellow),
            completed_style: Style::default().fg(Color::Green),
            failed_style: Style::default().fg(Color::Red),
            cancelled_style: Style::default().fg(Color::Gray),
            
            // Backend status styles
            healthy_style: Style::default().fg(Color::Green),
            warning_style: Style::default().fg(Color::Yellow),
            critical_style: Style::default().fg(Color::Red),
            offline_style: Style::default().fg(Color::DarkGray),
            
            // Chart and graph styles
            sparkline_style: Style::default().fg(Color::Green),
        }
    }
}


