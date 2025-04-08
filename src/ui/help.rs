//! Help overlay showing keyboard shortcuts and commands.

use ratatui::Frame;
use ratatui::layout::{Rect, Alignment};
use ratatui::widgets::{Block, Borders, Paragraph, Clear};
use ratatui::style::{Style, Modifier};
use ratatui::text::{Line, Span};

use crate::state::AppState;
use crate::ui::{Theme, ViewState};

/// Help overlay showing keyboard shortcuts and usage information.
pub struct HelpView;

impl HelpView {
    /// Render the help overlay
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        _app_state: &AppState,
        theme: &Theme,
        _current_view: &ViewState,
    ) {
        // Create a centered popup area that's 80% of the screen
        let popup_area = Self::centered_rect(60, 70, area);
        
        // Clear the background
        frame.render_widget(Clear, popup_area);
        
        // Create block for help text
        let help_block = Block::default()
            .title("Crankshaft TUI Help")
            .borders(Borders::ALL)
            .style(theme.block_style);
            
        // Prepare help text - both global shortcuts and context-specific ones
        let help_text = vec![
            Line::from(vec![
                Span::styled("Global Shortcuts", Style::default().add_modifier(Modifier::BOLD))
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("q", theme.key_style),
                Span::raw(" - Quit application"),
            ]),
            Line::from(vec![
                Span::styled("?", theme.key_style),
                Span::raw(" - Toggle this help screen"),
            ]),
            Line::from(vec![
                Span::styled("d", theme.key_style),
                Span::raw(" - Dashboard view"),
            ]),
            Line::from(vec![
                Span::styled("t", theme.key_style),
                Span::raw(" - Tasks list view"),
            ]),
            Line::from(vec![
                Span::styled("b", theme.key_style),
                Span::raw(" - Backends list view"),
            ]),
            Line::from(vec![
                Span::styled("p", theme.key_style),
                Span::raw(" - Toggle pause"),
            ]),
        ];
        
        // Create paragraph with help text
        let help_widget = Paragraph::new(help_text)
            .block(help_block)
            .style(theme.normal_text)
            .alignment(Alignment::Left);
            
        frame.render_widget(help_widget, popup_area);
    }
    
    /// Helper function to create a centered rect using percentages
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        // Calculate the size of the popup
        let popup_width = r.width * percent_x / 100;
        let popup_height = r.height * percent_y / 100;
        
        // Calculate the position
        let popup_x = (r.width - popup_width) / 2;
        let popup_y = (r.height - popup_height) / 2;
        
        // Create the rect
        Rect {
            x: r.x + popup_x,
            y: r.y + popup_y,
            width: popup_width,
            height: popup_height,
        }
    }
}
