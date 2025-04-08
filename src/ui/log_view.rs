//! Log view component for displaying application logs.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::Line;

use crate::state::AppState;
use crate::ui::Theme;

/// View for displaying application logs.
pub struct LogView;

impl LogView {
    /// Render the log view
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        app_state: &AppState,
        theme: &Theme,
    ) {
        // Create a block for the logs
        let block = Block::default()
            .title("Application Logs")
            .borders(Borders::ALL)
            .style(theme.block_style);
        
        // Format the logs (placeholder - you'd get real logs in a full implementation)
        let logs = vec![
            Line::from("Log output will appear here."),
            Line::from("Use this view to monitor application events."),
        ];
        
        // Create the paragraph widget with the logs
        let logs_widget = Paragraph::new(logs)
            .block(block)
            .style(theme.normal_text);
            
        // Render the widget
        frame.render_widget(logs_widget, area);
    }
}
