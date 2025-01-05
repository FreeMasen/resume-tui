use list_state::ListStateWrapper as ListState;
use ratatui::{
    prelude::*,
    widgets::{block::Title, *},
};

mod data;
mod detail_view;
mod edu;
mod list_state;
mod markdown;
mod oss;
mod work;

pub use data::source::DATABASE;

const DEFAULT_STYLE: Style = Style::new().fg(Color::Green).bg(Color::Black);

#[derive(Debug, Clone)]
pub struct App<'a> {
    main_menu_state: ListState,
    sub_page: Option<Page<'a>>,
}

pub trait Navigable {
    fn increment_selection(&mut self);
    fn decrement_selection(&mut self);
    fn handle_enter(&mut self);
    fn handle_left(&mut self) -> bool;
}

#[derive(Debug, Clone)]
enum Page<'a> {
    Work(work::WorkView<'a>),
    Oss(oss::OssView<'a>),
    Edu(edu::EduView),
}

impl<'a> Navigable for Page<'a> {
    fn increment_selection(&mut self) {
        match self {
            Page::Work(inner) => inner.increment_selection(),
            Page::Oss(inner) => inner.increment_selection(),
            Page::Edu(inner) => inner.increment_selection(),
        }
    }
    fn decrement_selection(&mut self) {
        match self {
            Page::Work(inner) => inner.decrement_selection(),
            Page::Oss(inner) => inner.decrement_selection(),
            Page::Edu(inner) => inner.decrement_selection(),
        }
    }

    fn handle_enter(&mut self) {
        match self {
            Page::Work(inner) => inner.handle_enter(),
            Page::Oss(inner) => inner.handle_enter(),
            Page::Edu(inner) => inner.handle_enter(),
        }
    }

    fn handle_left(&mut self) -> bool {
        match self {
            Page::Work(inner) => inner.handle_left(),
            Page::Oss(inner) => inner.handle_left(),
            Page::Edu(inner) => inner.handle_left(),
        }
    }
}

impl<'a> Default for App<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        Self {
            main_menu_state: ListState::new(4),
            sub_page: None,
        }
    }
    pub fn tick(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<(), Error> {
        self.draw(terminal)?;
        Ok(())
    }

    pub fn event(&mut self, event: Event) -> Result<(), Error> {
        match event {
            Event::Up => self.decrement_selection(),
            Event::Down => self.increment_selection(),
            Event::Left => self.handle_left(),
            Event::Right => self.handle_right(),
            Event::Quit => return Err(Error::Exit),
        }
        Ok(())
    }

    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> std::io::Result<()> {
        terminal.draw(|f| f.render_widget(self, f.area()))?;
        Ok(())
    }

    fn get_selected_menu_name(&self) -> &'static str {
        Self::menu_name(self.main_menu_state.selected().unwrap_or(255))
    }

    fn menu_name(idx: usize) -> &'static str {
        match idx {
            0 => "Home",
            1 => "Work",
            2 => "Open Source",
            3 => "Education",
            _ => "???",
        }
    }

    fn render_menu(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title("Menu")
            .title_alignment(Alignment::Center)
            .border_set(ratatui::symbols::border::PLAIN)
            .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
            .style(DEFAULT_STYLE);
        let content_area = block.inner(area);
        block.render(area, buf);
        let list = List::new([
            ListItem::from("Home"),
            "Work".into(),
            "Open Source".into(),
            "Education".into(),
        ]);
        let list = if self.sub_page.is_some() {
            list.style(Style::new().add_modifier(Modifier::DIM))
                .highlight_style(DEFAULT_STYLE.bold().remove_modifier(Modifier::DIM))
        } else {
            list.highlight_style(Style::new().bg(Color::Green).fg(Color::Black))
        };
        StatefulWidget::render(list, content_area, buf, self.main_menu_state.as_mut());
    }

    fn render_page(&mut self, area: Rect, buf: &mut Buffer) {
        let title = if self.sub_page.is_none() {
            "Home".to_string()
        } else {
            format!("{} - {}", DATABASE.name, self.get_selected_menu_name())
        };
        let total_area = Block::bordered()
            .title(Title::from(title))
            .title_alignment(Alignment::Center)
            .style(DEFAULT_STYLE)
            .border_set(ratatui::symbols::border::Set {
                top_left: symbols::line::NORMAL.horizontal_down,
                bottom_left: symbols::line::NORMAL.horizontal_up,
                ..symbols::border::PLAIN
            })
            .padding(Padding::ZERO);
        let inner_rect = total_area.inner(area);
        total_area.render(area, buf);
        let Some(sub_page) = self.sub_page.clone() else {
            self.render_home(inner_rect, buf);
            return;
        };
        match sub_page {
            Page::Work(work_state) => work_state.render(inner_rect, buf),
            Page::Oss(inner) => inner.render(inner_rect, buf),
            Page::Edu(inner) => inner.render(inner_rect, buf),
        }
    }

    fn render_home(&self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical(Constraint::from_percentages([45, 15, 40, 5]))
            .flex(layout::Flex::Center);
        let [_, content_area, _, footer] = layout.areas(area);

        Paragraph::new(vec![DATABASE.name.bold().into(), DATABASE.tag_line.into()])
            .alignment(Alignment::Center)
            .render(content_area, buf);
        let foot_layout = Layout::horizontal(Constraint::from_percentages([50, 50]));
        let [lhs, rhs] = foot_layout.areas(footer);
        Paragraph::new(
            DATABASE
                .github
                .map(|gh| format!(" https://github.com/{gh}"))
                .unwrap_or_default(),
        )
        .alignment(Alignment::Left)
        .render(lhs, buf);
        Paragraph::new(
            DATABASE
                .github
                .map(|li| format!("https://www.linkedin.com/in/{li} "))
                .unwrap_or_default(),
        )
        .alignment(Alignment::Right)
        .render(rhs, buf);
    }

    fn increment_selection(&mut self) {
        let Some(sub_page) = self.sub_page.as_mut() else {
            self.main_menu_state.increment();
            return;
        };
        sub_page.increment_selection();
    }

    fn decrement_selection(&mut self) {
        let Some(sub_page) = self.sub_page.as_mut() else {
            self.main_menu_state.decrement();
            return;
        };
        sub_page.decrement_selection();
    }

    fn handle_left(&mut self) {
        if let Some(mut page) = self.sub_page.take() {
            if page.handle_left() {
                self.sub_page = Some(page);
            }
        }
    }

    fn handle_right(&mut self) {
        if let Some(sub_page) = self.sub_page.as_mut() {
            sub_page.handle_enter();
            return;
        };
        let Some(selected) = self.main_menu_state.selected() else {
            return;
        };
        self.sub_page = match selected {
            0 => None,
            1 => Some(Page::Work(Default::default())),
            2 => Some(Page::Oss(Default::default())),
            3 => Some(Page::Edu(Default::default())),
            _ => return,
        }
    }
}

impl<'a> Widget for &mut App<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let full = Layout::horizontal([Constraint::Length(12), Constraint::Min(1)]);
        let [menu_area, display_area] = full.areas(area);

        self.render_menu(menu_area, buf);
        self.render_page(display_area, buf);
    }
}

pub enum Event {
    Up,
    Down,
    Left,
    Right,
    Quit,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Exit")]
    Exit,
}
