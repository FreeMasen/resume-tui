use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, List, ListItem, Paragraph, StatefulWidget, Widget},
};

use crate::{
    data::{Detail, Workplace, WORK},
    list_state::ListStateWrapper as ListState,
    Navigable,
};

#[derive(Debug, Clone)]
pub struct WorkPage {
    menu: ListState,
    work: Option<JobPage>,
}

impl Default for WorkPage {
    fn default() -> Self {
        Self {
            menu: ListState::new(WORK.len().saturating_sub(1)),
            work: None,
        }
    }
}

impl Widget for WorkPage {
    fn render(mut self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if let Some(sub_page) = self.work {
            sub_page.render(area, buf);
            return;
        }
        let list_items: Vec<ListItem> = WORK
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
            .highlight_style(Style::new().bg(Color::White).fg(Color::Green))
            .fg(Color::Green)
            .bg(Color::Black);
        StatefulWidget::render(list, area, buf, self.menu.as_mut());
    }
}

impl Navigable for WorkPage {
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
        self.work = WORK.get(idx).cloned().map(Into::into);
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
pub struct JobPage {
    workplace: Workplace,
    menu: ListState,
    detail: Option<DetailView<'static>>,
}

impl From<Workplace> for JobPage {
    fn from(value: Workplace) -> Self {
        let menu = ListState::new(value.details.len().saturating_sub(1));
        Self {
            workplace: value,
            menu,
            detail: None,
        }
    }
}

impl Navigable for JobPage {
    fn increment_selection(&mut self) {
        if self.detail.is_none() {
            self.menu.increment();
        }
    }

    fn decrement_selection(&mut self) {
        if self.detail.is_none() {
            self.menu.decrement();
        }
    }

    fn handle_enter(&mut self) {
        let Some(idx) = self.menu.selected() else {
            return;
        };
        let (pre_idx, should_set) = idx.overflowing_sub(1);
        if should_set {
            self.detail = self.workplace.details.iter().skip(pre_idx).next().map(Into::into);
        } else {
            self.detail = None;
        }
    }

    fn handle_left(&mut self) -> bool {
        if self.detail.take().is_some() {
            return true;
        }
        false
    }
}

impl Widget for JobPage {
    fn render(mut self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if let Some(sub_page) = self.detail {
            sub_page.render(area, buf);
            return;
        }
        let [header, dates, details] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .flex(Flex::Start)
        .areas(area);
        render_two_blocks(
            header,
            buf,
            [
                ("Company", self.workplace.name),
                ("Title", self.workplace.title),
            ]
            .into_iter(),
        );
        render_two_blocks(
            dates,
            buf,
            [
                ("Start", self.workplace.start),
                ("End", self.workplace.end.unwrap_or("Current")),
            ]
            .into_iter(),
        );
        render_job_details(&mut self.menu, self.workplace.details.iter(), details, buf);
    }
}

#[derive(Debug, Clone)]
struct DetailView<'a>(Text<'a>);

impl<'a> From<&'a Detail> for DetailView<'a> {
    fn from(detail: &'a Detail) -> Self {
        Self(Text::from(vec![
            Line::from(detail.short.bold()),
            Line::from(detail.long),
        ]))
    }
}

impl<'a> Widget for DetailView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.0.render(area, buf);
    }
}

fn render_two_blocks<'a>(
    area: Rect,
    buf: &mut Buffer,
    details: impl Iterator<Item = (&'a str, &'a str)>,
) {
    let cells: [Rect; 2] = Layout::horizontal(Constraint::from_percentages([50; 2])).areas(area);
    for (cell, (title, content)) in cells.into_iter().zip(details.into_iter()) {
        render_block(cell, buf, title, content);
    }
}

fn render_block<'a>(area: Rect, buf: &mut Buffer, title: &'a str, content: &'a str) {
    let block = Block::bordered()
        .title(title)
        .title_alignment(Alignment::Left);
    let rect = block.inner(area);
    Paragraph::new(content).render(rect, buf);
    block.render(area, buf);
}

fn render_job_details<'a>(
    state: &mut ListState,
    details: impl Iterator<Item = &'a Detail>,
    area: Rect,
    buf: &mut Buffer,
) {
    let block = Block::bordered().title("Details");
    let block_inner = block.inner(area);
    block.render(area, buf);
    let list: Vec<_> = details.into_iter().map(map_detail_to_list_item).collect();
    StatefulWidget::render(
        List::new(list).highlight_style(Style::default().fg(Color::Green).bg(Color::White)),
        block_inner,
        buf,
        state.as_mut(),
    );
}

fn map_detail_to_list_item<'a>(detail: &'a Detail) -> ListItem<'a> {
    let title = Line::from(detail.short.bold());
    let details = Line::from(detail.long);
    let text = Text::from(vec![title, details]);
    ListItem::new(text)
}
