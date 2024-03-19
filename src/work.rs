use ratatui::{
    buffer::Buffer, layout::{Alignment, Constraint, Flex, Layout, Rect}, style::{Color, Modifier, Style, Stylize}, symbols::{self, border::Set}, text::{Line, Text}, widgets::{
        Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget
    }
};

use crate::{
    data::{source::DATABASE, Detail, Workplace},
    list_state::ListStateWrapper as ListState,
    markdown::convert_md,
    Navigable, DEFAULT_STYLE,
};

#[derive(Debug, Clone)]
pub struct WorkView {
    menu: ListState,
    work: Option<JobView>,
}

impl Default for WorkView {
    fn default() -> Self {
        Self {
            menu: ListState::new(DATABASE.jobs.len().saturating_sub(1)),
            work: None,
        }
    }
}

impl Widget for WorkView {
    fn render(mut self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if let Some(sub_page) = self.work {
            sub_page.render(area, buf);
            return;
        }
        let list_items: Vec<ListItem> = DATABASE
            .jobs
            .iter()
            .map(|w| {
                let dts = if let Some(end) = w.end.as_ref() {
                    format!("{} - {end}", w.start)
                } else {
                    format!("{} - Current", w.start)
                };
                ListItem::new(Text::from(format!("{} - {}\n    {dts}", w.name, w.title,)))
            })
            .collect();
        let list = List::new(list_items)
            .highlight_style(Style::new().bg(Color::Green).fg(Color::Black))
            .style(DEFAULT_STYLE);
        StatefulWidget::render(list, area, buf, self.menu.as_mut());
    }
}

impl Navigable for WorkView {
    fn increment_selection(&mut self) {
        if let Some(sub_page) = &mut self.work {
            sub_page.increment_selection();
            return;
        }
        self.menu.increment();
    }

    fn decrement_selection(&mut self) {
        if let Some(sub_page) = &mut self.work {
            sub_page.decrement_selection();
            return;
        }
        self.menu.decrement();
    }

    fn handle_enter(&mut self) {
        if let Some(sub_page) = &mut self.work {
            sub_page.handle_enter();
            return;
        }
        let Some(idx) = self.menu.selected() else {
            return;
        };
        self.work = DATABASE.jobs.get(idx).cloned().map(Into::into);
    }

    fn handle_left(&mut self) -> bool {
        if let Some(mut sub_page) = self.work.take() {
            if sub_page.handle_left() {
                self.work = Some(sub_page);
            }
            return true;
        };
        false
    }
}

#[derive(Debug, Clone)]
pub struct JobView {
    workplace: Workplace,
    menu: ListState,
    detail: Option<DetailView>,
}

impl From<Workplace> for JobView {
    fn from(value: Workplace) -> Self {
        let menu = ListState::new(value.details.len().saturating_sub(1));
        Self {
            workplace: value,
            menu,
            detail: None,
        }
    }
}

impl Navigable for JobView {
    fn increment_selection(&mut self) {
        if let Some(detail) = self.detail.as_mut() {
            detail.increment_selection();
            return;
        }
        self.menu.increment();
    }

    fn decrement_selection(&mut self) {
        if let Some(detail) = self.detail.as_mut() {
            detail.decrement_selection();
            return;
        }
        self.menu.decrement();
    }

    fn handle_enter(&mut self) {
        log::trace!("JobPage::handle_enter");
        let Some(idx) = self.menu.selected() else {
            log::warn!("menu selected returned None");
            return;
        };
        self.detail = self
            .workplace
            .details
            .iter()
            .skip(idx)
            .next()
            .map(Into::into);
    }

    fn handle_left(&mut self) -> bool {
        if self.detail.take().is_some() {
            return true;
        }
        false
    }
}

impl Widget for JobView {
    fn render(mut self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if let Some(sub_page) = self.detail {
            sub_page.render(area, buf);
            return;
        }
        let [header, details] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .flex(Flex::Start)
        .areas(area);
        render_header(header, buf, [
            ("Company", self.workplace.name),
            ("Title", self.workplace.title),
            ("Start", self.workplace.start),
            ("End", self.workplace.end.unwrap_or("Current")),
        ].into_iter());
        render_job_details(&mut self.menu, self.workplace.details.iter(), details, buf);
    }
}

#[derive(Debug, Clone)]
struct DetailView {
    headline: &'static str,
    description: &'static str,
    scroll_state: ScrollbarState,
    scroll: u16,
    scroll_max: u16,
}

impl<'a> From<&'a Detail> for DetailView {
    fn from(detail: &'a Detail) -> Self {
        Self {
            headline: detail.headline,
            description: detail.detail,
            scroll_state: Default::default(),
            scroll: 0,
            scroll_max: 0,
        }
    }
}

impl Widget for DetailView {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let mut view =
            Text::from(vec![Line::from(self.headline)
                .style(DEFAULT_STYLE.bg(Color::Black).bold())]);

        let sb = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        view.lines
            .extend(convert_md(self.description, (area.width - 1) as _));
        
        let height = view.lines.len();
        let paragraph = Paragraph::new(view).scroll((self.scroll, 0));
        self.scroll_max = height as u16;
        self.scroll_state = self.scroll_state.content_length(height);
        paragraph.render(area, buf);
        StatefulWidget::render(sb, area, buf, &mut self.scroll_state);
    }
}

impl Navigable for DetailView {
    fn increment_selection(&mut self) {
        self.scroll = self.scroll.saturating_add(1).min(self.scroll_max);
        self.scroll_state = self.scroll_state.position(self.scroll as _);
    }

    fn decrement_selection(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
        self.scroll_state = self.scroll_state.position(self.scroll as _);
    }

    fn handle_enter(&mut self) {}

    fn handle_left(&mut self) -> bool {
        false
    }
}

fn render_header<'a>(
    area: Rect,
    buf: &mut Buffer,
    details: impl Iterator<Item = (&'static str, &'static str)>,
) {
    let borders = [
        (Borders::ALL ^ Borders::RIGHT, Set {
            top_right: symbols::line::NORMAL.horizontal_down,
            bottom_right: symbols::line::NORMAL.horizontal_up,
            ..Set::default()
        }),
        (Borders::ALL ^ Borders::RIGHT, Set {
            top_left: symbols::line::NORMAL.horizontal_down,
            bottom_left: symbols::line::NORMAL.horizontal_up,
            ..Set::default()
        }),
        (Borders::ALL ^ Borders::RIGHT, Set {
            top_left: symbols::line::NORMAL.horizontal_down,
            bottom_left: symbols::line::NORMAL.horizontal_up,
            ..Set::default()
        }),
        (Borders::ALL, Set {
            top_left: symbols::line::NORMAL.horizontal_down,
            bottom_left: symbols::line::NORMAL.horizontal_up,
            ..Set::default()
        }),
    ];
    let cells: [Rect; 4] = Layout::horizontal(Constraint::from_percentages([25; 4])).areas(area);
    for ((cell, (title, content)), (borders, set)) in cells.into_iter().zip(details.into_iter()).zip(borders.into_iter()) {
        render_header_block(cell, buf, title, content, borders, set);
    }
}

fn render_header_block<'a>(
    area: Rect,
    buf: &mut Buffer,
    title: &'static str,
    content: &'static str,
    border: Borders,
    corners: Set,
) {
    let block = Block::bordered()
        .title(title)
        .title_alignment(Alignment::Left)
        .style(DEFAULT_STYLE)
        .borders(border)
        .border_set(corners)
        .border_style(DEFAULT_STYLE);
    let rect = block.inner(area);
    block.render(area, buf);
    let content = convert_md(content, rect.width as usize);
    Paragraph::new(content).render(rect, buf);
}

fn render_job_details<'a>(
    state: &mut ListState,
    details: impl Iterator<Item = &'a Detail>,
    area: Rect,
    buf: &mut Buffer,
) {
    let list: Vec<_> = details.into_iter().map(map_detail_to_list_item).collect();
    StatefulWidget::render(
        List::new(list).highlight_style(Style::new().bg(Color::Green).fg(Color::Black)),
        area,
        buf,
        state.as_mut(),
    );
}

fn map_detail_to_list_item<'a>(detail: &'a Detail) -> ListItem<'a> {
    let title = Line::from(detail.headline.add_modifier(Modifier::BOLD));
    let details = Line::from(format!("  {}", detail.snippet));
    let text = Text::from(vec![title, details]);
    ListItem::new(text)
}
