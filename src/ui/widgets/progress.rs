//! Task progress visualization widget.
//!
//! This widget provides an enhanced progress bar for task completion tracking,
//! with features including:
//! - Context-aware styling based on progress percentage
//! - Efficient rendering with minimal allocations
//! - Status indicators and text overlays
//! - Customizable appearance

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};
use unicode_width::UnicodeWidthStr;

/// Enhanced progress bar widget for task completion visualization.
/// 
/// # Design Notes
/// This widget demonstrates the tokio-console approach of context-aware
/// rendering and style application, optimizing CPU and memory usage.
pub struct ProgressBar<'a> {
    /// Optional block to display around the progress bar
    block: Option<Block<'a>>,
    /// The value (0.0-1.0) representing progress
    progress: f64,
    /// Style for the filled portion of the progress bar
    style: Style,
    /// Style for the empty portion of the progress bar
    empty_style: Style,
    /// Symbol used for the filled part (default: █)
    symbol_filled: &'a str,
    /// Symbol used for the empty part (default: ░)
    symbol_empty: &'a str,
    /// Optional label to show in the center of the progress bar
    label: Option<String>,
    /// Whether to show percentage text
    show_percentage: bool,
    /// Optional style that changes based on progress percentage
    dynamic_style: bool,
    /// Animation frame for "in progress" indicators
    animation_frame: usize,
}

impl<'a> Default for ProgressBar<'a> {
    fn default() -> Self {
        Self {
            block: None,
            progress: 0.0,
            style: Style::default().fg(Color::Green),
            empty_style: Style::default().fg(Color::DarkGray),
            symbol_filled: "█",
            symbol_empty: "░",
            label: None,
            show_percentage: false,
            dynamic_style: false,
            animation_frame: 0,
        }
    }
}

impl<'a> ProgressBar<'a> {
    /// Create a new progress bar with the given value (0.0-1.0)
    pub fn new(progress: f64) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            ..Default::default()
        }
    }
    
    /// Set the block to display around the progress bar
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
    
    /// Set the style for the filled portion
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
    
    /// Set the style for the empty portion
    pub fn empty_style(mut self, style: Style) -> Self {
        self.empty_style = style;
        self
    }
    
    /// Set the label to display in the center
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    
    /// Toggle showing percentage text
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }
    
    /// Enable dynamic styling based on progress
    pub fn dynamic_style(mut self, enabled: bool) -> Self {
        self.dynamic_style = enabled;
        self
    }
    
    /// Set animation frame for progress indicators
    pub fn animation_frame(mut self, frame: usize) -> Self {
        self.animation_frame = frame;
        self
    }
    
    /// Get the style based on progress percentage when dynamic styling is enabled
    fn get_dynamic_style(&self) -> Style {
        if !self.dynamic_style {
            return self.style;
        }
        
        // Apply dynamic color based on progress
        match (self.progress * 100.0) as u8 {
            0..=30 => Style::default().fg(Color::Red),
            31..=60 => Style::default().fg(Color::Yellow),
            61..=90 => Style::default().fg(Color::Cyan),
            _ => Style::default().fg(Color::Green),
        }
    }
}

impl<'a> Widget for ProgressBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Skip rendering if there's not enough space
        if area.height < 1 {
            return;
        }
        
        // Render block if specified
        let render_area = if let Some(ref block) = self.block {
            let inner_area = block.inner(area);
            block.clone().render(area, buf); 
            inner_area
        } else {
            area
        };
        
        // Skip if there's not enough space after block rendering
        if render_area.width < 1 {
            return;
        }
        
        // Calculate the filled width based on progress
        let filled_width = ((render_area.width as f64) * self.progress).round() as u16;
        let style = self.get_dynamic_style();
        
        // Draw the filled portion
        for y in render_area.top()..render_area.bottom() {
            for x in render_area.left()..render_area.left().saturating_add(filled_width) {
                buf.get_mut(x, y).set_symbol(self.symbol_filled).set_style(style);
            }
            
            // Draw the empty portion
            for x in render_area.left().saturating_add(filled_width)..render_area.right() {
                buf.get_mut(x, y).set_symbol(self.symbol_empty).set_style(self.empty_style);
            }
        }
        
        // Render percentage or label - use the tokio-console approach of creating a temporary value
        // rather than allocating a new string when not necessary
        let center_text = if self.show_percentage {
            format!("{:3.0}%", self.progress * 100.0)
        } else if let Some(label) = &self.label {
            label.clone()
        } else {
            return;
        };
        
        // Skip if there's no space for the text
        if center_text.width() as u16 >= render_area.width {
            return;
        }
        
        // Render the text centered
        let text_x = render_area.left() + (render_area.width - center_text.width() as u16) / 2;
        let text_y = render_area.top();
        
        // Dynamic text styling for visibility
        let text_style = if self.progress > 0.5 {
            Style::default().fg(Color::Black).bg(style.fg.unwrap_or(Color::Green))
        } else {
            Style::default().fg(Color::White)
        };
        
        for (i, c) in center_text.chars().enumerate() {
            let x = text_x + i as u16;
            if x < render_area.right() {
                buf.get_mut(x, text_y).set_char(c).set_style(text_style);
            }
        }
    }
}