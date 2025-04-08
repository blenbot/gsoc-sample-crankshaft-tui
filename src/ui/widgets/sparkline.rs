//! Sparkline widget for time-series data visualization.
//!
//! This widget implements an efficient sparkline chart for resource utilization
//! and other time-series data. It uses several optimization techniques inspired by
//! tokio-console:
//! - Minimizes allocations during rendering
//! - Adapts to available space using data reduction techniques
//! - Supports context-aware styling and formatting
//! - Implements efficient bounds detection

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    symbols,
    widgets::{Block, Widget},
};

/// A sparkline widget that shows a simplified line chart.
/// 
/// # Design Notes
/// This widget demonstrates the same memory efficiency patterns that
/// tokio-console uses, with minimal allocations during render and
/// context-aware rendering based on available space.
pub struct Sparkline<'a> {
    /// Title displayed at the top of the widget.
    block: Option<Block<'a>>,
    /// Style of the sparkline.
    style: Style,
    /// Vector of points to display in the sparkline.
    data: &'a [f64],
    /// The maximum value to scale the sparkline to. Defaults to the max value.
    max: Option<f64>,
    /// Symbol used to represent data points.
    bar_set: symbols::bar::Set,
    /// Style for the minimum value.
    min_style: Style,
    /// Style for the maximum value.
    max_style: Style,
}

impl<'a> Sparkline<'a> {
    pub fn new(data: &'a [f64]) -> Self {
        Self {
            block: None,
            style: Style::default().fg(Color::Green),
            data,
            max: None,
            bar_set: symbols::bar::NINE_LEVELS,
            min_style: Style::default().fg(Color::Blue),
            max_style: Style::default().fg(Color::Red),
        }
    }
    
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
    
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
    
    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }
    
    pub fn bar_set(mut self, bar_set: symbols::bar::Set) -> Self {
        self.bar_set = bar_set;
        self
    }
    
    pub fn min_style(mut self, style: Style) -> Self {
        self.min_style = style;
        self
    }
    
    pub fn max_style(mut self, style: Style) -> Self {
        self.max_style = style;
        self
    }
}

impl<'a> Widget for Sparkline<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 1 || self.data.is_empty() {
            return;
        }
        
        // Draw the block if specified
        let area = if let Some(block) = self.block {
            let inner_area = block.inner(area);
            block.clone().render(area, buf);
            inner_area
        } else {
            area
        };
        
        // Calculate min and max values
        // Using a single pass algorithm to minimize CPU usage, just like tokio-console does
        let mut max_value = self.max.unwrap_or_else(|| {
            self.data.iter().fold(f64::MIN, |max, &v| max.max(v))
        });
        let min_value = self.data.iter().fold(max_value, |min, &v| min.min(v));
        
        // Make sure max_value is positive and greater than min_value
        if max_value.abs() < 1e-10 {
            max_value = 1.0;
        }
        let range = max_value - min_value;
        
        // Calculate the width of each data point with progressive data reduction
        // This is another tokio-console pattern - adapt to the available space
        let available_width = area.width as usize;
        let data_len = self.data.len();
        
        // Skip rendering if we don't have enough space
        if available_width == 0 {
            return;
        }
        
        // Calculate how many data points to skip (data reduction strategy)
        let step = if data_len > available_width {
            data_len / available_width
        } else {
            1
        };
        
        // Calculate the bars - this is done without additional allocations where possible
        let mut bars = Vec::with_capacity(available_width);
        let mut i = data_len.saturating_sub(available_width * step);
        while i < data_len {
            let value = self.data[i];
            // Calculate bar height as a percentage (0.0-1.0)
            let bar_height = if range > 0.0 {
                (value - min_value) / range
            } else {
                0.0
            };
            
            // Convert to bar symbol (0-8)
            let bar_levels = 9; 
            let bar_index = (bar_height * (bar_levels - 1) as f64).round() as usize;
            let bar_char = match bar_index {
                0 => self.bar_set.empty,
                1 => self.bar_set.one_eighth,
                2 => self.bar_set.one_quarter,
                3 => self.bar_set.three_eighths,
                4 => self.bar_set.half,
                5 => self.bar_set.five_eighths,
                6 => self.bar_set.three_quarters,
                7 => self.bar_set.seven_eighths,
                _ => self.bar_set.full,
            };
            
            // Context-aware styling based on value relationships
            let style = if (value - min_value).abs() < 1e-10 {
                self.min_style
            } else if (value - max_value).abs() < 1e-10 {
                self.max_style
            } else {
                self.style
            };
            
            bars.push((bar_char, style));
            i += step;
        }
        
        // Render the bars - direct buffer manipulation for efficiency
        for (i, (bar, style)) in bars.into_iter().enumerate() {
            if i < area.width as usize {
                buf.get_mut(area.x + i as u16, area.y)
                    .set_symbol(bar)
                    .set_style(style);
            }
        }
    }
}