//! Statistical panel widget for displaying metric values.
//!
//! This widget implements a panel for displaying statistics with labels,
//! values, and optional trends. It uses memory-efficient rendering and
//! conditional formatting based on value thresholds.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

/// Trend indicator for stats.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Trend {
    Up,
    Down,
    Neutral,
    None,
}

/// A value with associated metadata for context-aware rendering.
#[derive(Debug, Clone)]
pub struct StatValue {
    /// The main value to display
    value: String,
    /// Optional trend indicator
    trend: Trend,
    /// Whether this value represents a healthy state
    is_healthy: bool,
    /// Optional previous value (for comparison)
    previous: Option<String>,
    /// Optional threshold for warning
    warn_threshold: Option<f64>,
    /// Optional threshold for critical state
    crit_threshold: Option<f64>,
}

impl StatValue {
    /// Create a new stat value with the given string
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            trend: Trend::None,
            is_healthy: true,
            previous: None,
            warn_threshold: None,
            crit_threshold: None,
        }
    }
    
    /// Add a trend indicator to this stat
    pub fn trend(mut self, trend: Trend) -> Self {
        self.trend = trend;
        self
    }
    
    /// Mark the stat as healthy/unhealthy
    pub fn healthy(mut self, is_healthy: bool) -> Self {
        self.is_healthy = is_healthy;
        self
    }
    
    /// Add previous value for comparison
    pub fn previous(mut self, prev: impl Into<String>) -> Self {
        self.previous = Some(prev.into());
        self
    }
    
    /// Set warning threshold
    pub fn warn_at(mut self, threshold: f64) -> Self {
        self.warn_threshold = Some(threshold);
        self
    }
    
    /// Set critical threshold
    pub fn critical_at(mut self, threshold: f64) -> Self {
        self.crit_threshold = Some(threshold);
        self
    }
    
    /// Get appropriate color based on the value and thresholds
    fn color_for_value(&self, value_f64: Option<f64>) -> Color {
        if !self.is_healthy {
            return Color::Red;
        }
        
        if let Some(value) = value_f64 {
            if let Some(crit) = self.crit_threshold {
                if value >= crit {
                    return Color::Red;
                }
            }
            
            if let Some(warn) = self.warn_threshold {
                if value >= warn {
                    return Color::Yellow;
                }
            }
        }
        
        Color::Green
    }
    
    /// Get style for this stat
    fn get_style(&self) -> Style {
        // Try to parse value as f64 for threshold comparison
        let value_f64 = self.value.parse::<f64>().ok();
        
        Style::default().fg(self.color_for_value(value_f64))
    }
    
    /// Get trend indicator symbol
    fn trend_symbol(&self) -> &'static str {
        match self.trend {
            Trend::Up => "↑",
            Trend::Down => "↓",
            Trend::Neutral => "→",
            Trend::None => "",
        }
    }
    
    /// Get spans for rendering this stat
    fn to_spans(&self) -> Vec<Span<'static>> {
        let mut spans = Vec::with_capacity(3);
        
        // Add the value with appropriate style
        spans.push(Span::styled(
            self.value.clone(),
            self.get_style().add_modifier(Modifier::BOLD)
        ));
        
        // Add trend symbol if present
        let trend_symbol = self.trend_symbol();
        if !trend_symbol.is_empty() {
            spans.push(Span::styled(
                format!(" {}", trend_symbol),
                match self.trend {
                    Trend::Up => Style::default().fg(Color::Green),
                    Trend::Down => Style::default().fg(Color::Red),
                    _ => Style::default().fg(Color::Gray),
                }
            ));
        }
        
        // Add previous value for comparison if present
        if let Some(prev) = &self.previous {
            spans.push(Span::styled(
                format!(" (was: {})", prev),
                Style::default().fg(Color::DarkGray)
            ));
        }
        
        spans
    }
}

/// A panel showing statistics with labels and values.
pub struct StatPanel<'a> {
    /// Block surrounding the panel
    block: Option<Block<'a>>,
    /// List of labels and values
    stats: Vec<(&'a str, StatValue)>,
    /// Style for the labels
    label_style: Style,
    /// Whether to right-align values
    right_align: bool,
    /// Space between label and value
    spacing: usize,
}

impl<'a> StatPanel<'a> {
    /// Create a new stat panel
    pub fn new() -> Self {
        Self {
            block: None,
            stats: Vec::new(),
            label_style: Style::default().add_modifier(Modifier::BOLD),
            right_align: false,
            spacing: 2,
        }
    }
    
    /// Set the block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
    
    /// Add a statistic to the panel
    pub fn stat(mut self, label: &'a str, value: StatValue) -> Self {
        self.stats.push((label, value));
        self
    }
    
    /// Set the style for labels
    pub fn label_style(mut self, style: Style) -> Self {
        self.label_style = style;
        self
    }
    
    /// Set right alignment for values
    pub fn right_align(mut self, right_align: bool) -> Self {
        self.right_align = right_align;
        self
    }
    
    /// Set spacing between label and value
    pub fn spacing(mut self, spacing: usize) -> Self {
        self.spacing = spacing;
        self
    }
}

impl<'a> Widget for StatPanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 1 || self.stats.is_empty() {
            return;
        }
        
        // Render block if specified
        let render_area = if let Some(block) = self.block {
            let inner_area = block.inner(area);
            block.render(area, buf);
            inner_area
        } else {
            area
        };
        
        // Skip if there's not enough space after block rendering
        if render_area.height < 1 {
            return;
        }
        
        // Find the longest label for alignment
        let max_label_len = self.stats.iter()
            .map(|(label, _)| label.len())
            .max()
            .unwrap_or(0);
        
        // Create text content with each stat on its own line
        let mut text = Vec::with_capacity(self.stats.len());
        
        for (i, (label, value)) in self.stats.iter().enumerate() {
            // Skip if we've run out of vertical space
            if i >= render_area.height as usize {
                break;
            }
            
            let mut spans = Vec::with_capacity(3);
            
            // Add label with padding based on alignment
            let label_text = if self.right_align {
                format!("{:>width$}", label, width = max_label_len)
            } else {
                label.to_string()
            };
            
            spans.push(Span::styled(label_text, self.label_style));
            
            // Add separator
            spans.push(Span::raw(format!("{:spacing$}", "", spacing = self.spacing)));
            
            // Add value with appropriate styling
            spans.extend(value.to_spans());
            
            text.push(Line::from(spans));
        }
        
        // Render the paragraph
        let paragraph = Paragraph::new(text);
        paragraph.render(render_area, buf);
    }
}



