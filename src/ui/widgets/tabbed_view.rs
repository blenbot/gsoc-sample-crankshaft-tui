//! Tabbed view widget for multi-page interfaces.
//!
//! This widget implements a tabbed interface with content pages and tab navigation.
//! It uses a single rendering pass for efficiency and supports dynamic content
//! based on the currently selected tab.

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Tabs, Widget},
};

/// A tabbed view widget that shows different content based on selected tab.
///
/// # Design Notes
/// This widget implements the tokio-console pattern of componentization with
/// clear separation of concerns. It manages tab selection state internally
/// and delegates content rendering to tab-specific handlers.
pub struct TabbedView<'a, T> {
    /// Block surrounding the entire widget
    block: Option<Block<'a>>,
    /// Tab titles
    titles: Vec<&'a str>,
    /// Currently selected tab index
    selected: usize,
    /// Style for normal tabs
    tab_style: Style,
    /// Style for the selected tab
    selected_tab_style: Style,
    /// Content to display for each tab
    content: T,
}

impl<'a, T> TabbedView<'a, T> {
    /// Create a new tabbed view
    pub fn new(content: T) -> Self {
        Self {
            block: None,
            titles: Vec::new(),
            selected: 0,
            tab_style: Style::default(),
            selected_tab_style: Style::default().add_modifier(Modifier::BOLD),
            content,
        }
    }
    
    /// Set the surrounding block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
    
    /// Add tab titles
    pub fn titles(mut self, titles: Vec<&'a str>) -> Self {
        self.titles = titles;
        self
    }
    
    /// Set the selected tab index
    pub fn select(mut self, index: usize) -> Self {
        self.selected = index.min(self.titles.len().saturating_sub(1));
        self
    }
    
    /// Set style for normal tabs
    pub fn tab_style(mut self, style: Style) -> Self {
        self.tab_style = style;
        self
    }
    
    /// Set style for selected tab
    pub fn selected_tab_style(mut self, style: Style) -> Self {
        self.selected_tab_style = style;
        self
    }
}

impl<'a, T: TabRenderer> Widget for TabbedView<'a, T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 3 || self.titles.is_empty() {
            return;
        }
        
        // Render block if specified
        let area = if let Some(block) = self.block {
            let inner_area = block.inner(area);
            block.render(area, buf);
            inner_area
        } else {
            area
        };
        
        // Calculate layout for tabs and content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Height for tabs
                Constraint::Min(1),    // Rest for content
            ])
            .split(area);
            
        // Create styled tab titles
        let titles: Vec<Line> = self.titles
            .iter()
            .map(|t| {
                let style = if self.titles[self.selected] == *t {
                    self.selected_tab_style
                } else {
                    self.tab_style
                };
                Line::from(Span::styled(*t, style))
            })
            .collect();
            
        // Render the tabs
        let tabs = Tabs::new(titles)
            .select(self.selected)
            .divider("|");
            
        tabs.render(chunks[0], buf);
        
        // Render the content using the content handler
        self.content.render_tab_content(self.selected, chunks[1], buf);
    }
}

/// Trait for rendering tab content
pub trait TabRenderer {
    /// Render content for the specific tab
    fn render_tab_content(&self, tab_index: usize, area: Rect, buf: &mut Buffer);
}

// Simple implementation for closure-based content rendering
impl<F> TabRenderer for F 
where
    F: Fn(usize, Rect, &mut Buffer),
{
    fn render_tab_content(&self, tab_index: usize, area: Rect, buf: &mut Buffer) {
        self(tab_index, area, buf);
    }
}
