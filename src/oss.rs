use ratatui::{
    buffer::Buffer, layout::{Alignment, Constraint, Layout, Rect}, style::{Color, Style}, symbols::{self, border::Set}, text::Text, widgets::{Block, Borders, List, ListItem, Paragraph, StatefulWidget, Widget}
};

use crate::{
    data::{source::DATABASE, Project},
    list_state::ListStateWrapper as ListState,
    markdown::convert_md,
    Navigable, DEFAULT_STYLE,
};

#[derive(Debug, Clone)]
pub struct OssView {
    menu: ListState,
    sub_page: Option<ProjectView>,
}

impl Default for OssView {
    fn default() -> Self {
        Self {
            menu: ListState::new(DATABASE.open_source.len().saturating_sub(1)),
            sub_page: None,
        }
    }
}

impl Widget for OssView {
    fn render(mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if let Some(sub_page) = self.sub_page {
            sub_page.render(area, buf);
            return;
        }
        let list_items: Vec<ListItem> = DATABASE
            .open_source
            .iter()
            .map(|w| ListItem::new(Text::from(format!("{}\n    {}", w.name, w.short_desc,))))
            .collect();
        let list = List::new(list_items)
            .highlight_style(Style::new().bg(Color::Green).fg(Color::Black))
            .style(DEFAULT_STYLE);
        StatefulWidget::render(list, area, buf, self.menu.as_mut());
    }
}

impl Navigable for OssView {
    fn increment_selection(&mut self) {
        if let Some(sub_page) = self.sub_page.as_mut() {
            sub_page.increment_selection();
            return;
        }
        self.menu.increment()
    }

    fn decrement_selection(&mut self) {
        if let Some(sub_page) = self.sub_page.as_mut() {
            sub_page.decrement_selection();
            return;
        }
        self.menu.decrement()
    }

    fn handle_enter(&mut self) {
        if let Some(sub_page) = self.sub_page.as_mut() {
            sub_page.handle_enter();
            return;
        }
        let Some(idx) = self.menu.selected() else {
            return;
        };
        self.sub_page = DATABASE.open_source.get(idx).map(Into::into);
    }

    fn handle_left(&mut self) -> bool {
        if let Some(mut sub_page) = self.sub_page.take() {
            if sub_page.handle_left() {
                self.sub_page = Some(sub_page);
            }
            return true;
        }
        false
    }
}

#[derive(Debug, Clone)]
pub struct ProjectView {
    project: Project,
    menu: ListState,
    sub_page: Option<SubProjectView>,
}

#[derive(Debug, Clone)]
pub enum SubProjectView {
    SubProject(Box<ProjectView>),
    LongDescription(&'static str),
}

impl From<&Project> for ProjectView {
    fn from(value: &Project) -> Self {
        Self {
            project: value.clone(),
            menu: ListState::new(value.sub_projects.len() + 1),
            sub_page: None,
        }
    }
}

impl Widget for ProjectView {
    fn render(mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if let Some(sub_page) = self.sub_page.take() {
            sub_page.render(area, buf);
            return;
        }
        let [header, details] = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .areas(area);

        render_two_blocks(
            header,
            buf,
            [
                ("project", self.project.name),
                ("desc", self.project.short_desc),
            ]
            .into_iter(),
        );

        if self.project.sub_projects.is_empty() {
            render_block(
                details,
                buf,
                "Detailed Description",
                self.project.long_desc,
                Borders::ALL,
                Default::default(),
            );
            return;
        }
        let mut items = vec![
            ListItem::new("Detailed Description"),
            ListItem::new("Projects"),
        ];
        items.extend(
            self.project
                .sub_projects
                .into_iter()
                .map(|p| ListItem::new(format!("  {}", p.name))),
        );
        StatefulWidget::render(
            List::new(items)
                .highlight_style(Style::new().bg(Color::Green).fg(Color::Black))
                .style(DEFAULT_STYLE),
            details,
            buf,
            self.menu.as_mut(),
        );
    }
}

impl Widget for SubProjectView {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        match self {
            Self::LongDescription(text) => convert_md(text, area.width as usize).render(area, buf),
            Self::SubProject(proj) => (*proj).render(area, buf),
        }
    }
}

impl Navigable for ProjectView {
    fn increment_selection(&mut self) {
        log::trace!("ProjectView::increment_selection");
        let Some(sub_page) = self.sub_page.as_mut() else {
            if self.project.sub_projects.is_empty() {
                return;
            }
            self.menu.increment();
            if self.menu.selected() == Some(1) {
                self.menu.increment();
            }
            return;
        };
        let SubProjectView::SubProject(inner) = sub_page else {
            return;
        };
        inner.increment_selection();
    }

    fn decrement_selection(&mut self) {
        let Some(sub_page) = self.sub_page.as_mut() else {
            if self.project.sub_projects.is_empty() {
                return;
            }
            self.menu.decrement();
            if self.menu.selected() == Some(1) {
                self.menu.decrement();
            }
            return;
        };
        let SubProjectView::SubProject(inner) = sub_page else {
            return;
        };
        inner.decrement_selection();
    }

    fn handle_enter(&mut self) {
        if let Some(sub_page) = self.sub_page.as_mut() {
            if let SubProjectView::SubProject(sp) = sub_page {
                sp.handle_enter();
            }
            return;
        }
        let Some(idx) = self.menu.selected() else {
            return;
        };
        if idx == 0 {
            self.sub_page = Some(SubProjectView::LongDescription(self.project.long_desc));
        } else if let Some(sub_project) = self.project.sub_projects.get(idx - 2).cloned() {
            self.sub_page = Some(SubProjectView::SubProject(Box::new(ProjectView {
                menu: ListState::new(sub_project.sub_projects.len()),
                project: sub_project,
                sub_page: None,
            })))
        }
    }

    fn handle_left(&mut self) -> bool {
        if let Some(mut sub_page) = self.sub_page.take() {
            if sub_page.handle_left() {
                self.sub_page = Some(sub_page);
            }
            return true;
        }
        false
    }
}

impl Navigable for SubProjectView {
    fn increment_selection(&mut self) {
        let SubProjectView::SubProject(inner) = self else {
            return;
        };
        inner.increment_selection()
    }

    fn decrement_selection(&mut self) {
        let SubProjectView::SubProject(inner) = self else {
            return;
        };
        inner.decrement_selection()
    }

    fn handle_enter(&mut self) {
        let SubProjectView::SubProject(inner) = self else {
            return;
        };
        inner.handle_enter()
    }

    fn handle_left(&mut self) -> bool {
        let SubProjectView::SubProject(inner) = self else {
            return false;
        };
        inner.handle_left()
    }
}

fn render_two_blocks<'a>(
    area: Rect,
    buf: &mut Buffer,
    details: impl Iterator<Item = (&'static str, &'static str)>,
) {
    let borders = [
            (Borders::ALL, Set {
                top_right: symbols::line::NORMAL.horizontal_down,
                bottom_right: symbols::line::NORMAL.horizontal_up,
                ..Default::default()
            }),
            (Borders::ALL ^ Borders::LEFT, Set {
                top_right: symbols::line::NORMAL.horizontal_down,
                bottom_right: symbols::line::NORMAL.horizontal_up,
                ..Default::default()
            }),
        ];
    let cells: [Rect; 2] = Layout::horizontal(Constraint::from_percentages([50; 2])).areas(area);
    for ((cell, (title, content)), (borders, set)) in cells.into_iter().zip(details.into_iter()).zip(borders.into_iter()) {
        render_block(cell, buf, title, content, borders, set);
    }
}

fn render_block(area: Rect, buf: &mut Buffer, title: &'static str, content: &'static str, border: Borders, set: Set) {
    let block = Block::bordered()
        .title(title)
        .title_alignment(Alignment::Left)
        .border_set(set)
        .borders(border)
        .style(DEFAULT_STYLE);
    let rect = block.inner(area);
    block.render(area, buf);
    let content = crate::markdown::convert_md(content, rect.width as usize);
    Paragraph::new(content).render(rect, buf);
}
