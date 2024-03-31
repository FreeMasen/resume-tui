use std::sync::{atomic::AtomicUsize, Arc};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Modifier,
    text::Text,
    widgets::{
        Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
        Widget, Wrap,
    },
};

use crate::{data::Detail, markdown::convert_md, Navigable, DEFAULT_STYLE};

#[derive(Debug, Clone)]
pub struct DetailView<'a> {
    title: &'static str,
    content: Text<'a>,
    scroll: usize,
    scroll_max: Arc<AtomicUsize>,
}

impl<'a> DetailView<'a> {
    pub fn new(title: &'static str, content: &'static str) -> Self {
        Self {
            title,
            content: convert_md(content),
            scroll: 0,
            scroll_max: Arc::new(AtomicUsize::new(100)),
        }
    }
}

impl<'a> From<&'a Detail> for DetailView<'a> {
    fn from(detail: &'a Detail) -> Self {
        Self::new(detail.headline, detail.detail)
    }
}

impl<'a> Widget for DetailView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [title, detail] =
            Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(area);
        let block = Block::new()
            .borders(Borders::BOTTOM)
            .border_style(DEFAULT_STYLE.add_modifier(Modifier::BOLD));

        let [content, scroll_bar] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)]).areas(detail);
        let text = Text::raw(self.title).style(DEFAULT_STYLE.add_modifier(Modifier::BOLD));
        let block_area = block.inner(title);
        block.render(title, buf);
        text.render(block_area, buf);
        let height =
            calc_lines(&self.content, content.width as _).saturating_sub(content.height as _);
        self.scroll_max
            .store(height, std::sync::atomic::Ordering::Relaxed);
        let para: Paragraph<'_> = Paragraph::new(self.content)
            .wrap(Wrap { trim: false })
            .scroll(((self.scroll as u16).min(height as _), 0));
        para.render(content, buf);
        let scroll = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let mut scroll_state = ScrollbarState::new(height)
            .viewport_content_length(content.height as _)
            .position(self.scroll);
        StatefulWidget::render(scroll, scroll_bar, buf, &mut scroll_state);
    }
}

impl<'a> Navigable for DetailView<'a> {
    fn increment_selection(&mut self) {
        log::trace!("DetailView::increment_selection");
        let max = self.scroll_max.load(std::sync::atomic::Ordering::Relaxed);
        log::debug!("max: {max}, scroll: {}", self.scroll);
        self.scroll = self.scroll.saturating_add(1).min(max);
    }

    fn decrement_selection(&mut self) {
        log::trace!("DetailView::decrement_selection");
        self.scroll = self.scroll.saturating_sub(1);
    }

    fn handle_enter(&mut self) {}

    fn handle_left(&mut self) -> bool {
        false
    }
}

fn calc_lines(text: &Text, view_width: usize) -> usize {
    let mut ret = 0;
    for line in &text.lines {
        ret += 1;
        let mut current_width = 0;
        for span in &line.spans {
            current_width += span.width();
            if current_width >= view_width {
                ret += 1;
                current_width -= view_width;
            }
        }
    }
    ret
}
