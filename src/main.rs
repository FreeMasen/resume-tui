use color_eyre::config::HookBuilder;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use list_state::ListStateWrapper as ListState;
use ratatui::{
    prelude::*,
    widgets::{block::Title, *},
};

mod data;
mod list_state;
mod markdown;
mod oss;
mod work;

use data::source::DATABASE;

#[derive(Debug, Clone)]
struct App {
    main_menu_state: ListState,
    sub_page: Option<Page>,
}

pub trait Navigable {
    fn increment_selection(&mut self);
    fn decrement_selection(&mut self);
    fn handle_enter(&mut self);
    fn handle_left(&mut self) -> bool;
}

#[derive(Debug, Clone)]
enum Page {
    Work(work::WorkPage),
    Oss(oss::OssView),
}

impl Navigable for Page {
    fn increment_selection(&mut self) {
        match self {
            Page::Work(inner) => inner.increment_selection(),
            Page::Oss(inner) => inner.increment_selection(),
        }
    }
    fn decrement_selection(&mut self) {
        match self {
            Page::Work(inner) => inner.decrement_selection(),
            Page::Oss(inner) => inner.decrement_selection(),
        }
    }

    fn handle_enter(&mut self) {
        match self {
            Page::Work(inner) => inner.handle_enter(),
            Page::Oss(inner) => inner.handle_enter(),
        }
    }

    fn handle_left(&mut self) -> bool {
        match self {
            Page::Work(inner) => inner.handle_left(),
            Page::Oss(inner) => inner.handle_left(),
        }
    }
}

impl App {
    fn new() -> Self {
        Self {
            main_menu_state: ListState::new(3),
            sub_page: None,
        }
    }
    fn run(&mut self, mut terminal: Terminal<impl Backend>) -> std::io::Result<()> {
        loop {
            self.draw(&mut terminal)?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('h') | KeyCode::Left => self.handle_left(),
                        KeyCode::Char('j') | KeyCode::Down => self.increment_selection(),
                        KeyCode::Char('k') | KeyCode::Up => {
                            self.decrement_selection();
                        }
                        KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                            if let Some(sub_page) = self.sub_page.as_mut() {
                                sub_page.handle_enter();
                                continue;
                            };
                            let Some(selected) = self.main_menu_state.selected() else {
                                continue;
                            };
                            self.sub_page = match selected {
                                0 => None,
                                1 => Some(Page::Work(Default::default())),
                                2 => Some(Page::Oss(Default::default())),
                                _ => continue,
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> std::io::Result<()> {
        terminal.draw(|f| f.render_widget(self, f.size()))?;
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
            .style(Style::new().fg(Color::Green).bg(Color::Black));
        let content_area = block.inner(area);
        block.render(area, buf);
        let list = List::new([ListItem::from("Home"), "Work".into(), "Open Source".into()]);
        StatefulWidget::render(
            list.highlight_style(Style::new().bg(Color::White).fg(Color::Green)),
            content_area,
            buf,
            self.main_menu_state.as_mut(),
        );
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
            .style(Style::new().fg(Color::Green).bg(Color::Black));
        let inner_rect = total_area.inner(area);
        total_area.render(area, buf);
        let Some(sub_page) = self.sub_page.clone() else {
            self.render_home(inner_rect, buf);
            return;
        };
        match sub_page {
            Page::Work(work_state) => work_state.render(inner_rect, buf),
            Page::Oss(inner) => inner.render(inner_rect, buf),
        }
    }

    fn render_home(&self, area: Rect, buf: &mut Buffer) {
        let layout =
            Layout::vertical(Constraint::from_percentages([45, 10, 45])).flex(layout::Flex::Center);
        let [_, content_area, _] = layout.areas(area);
        Paragraph::new(vec![DATABASE.name.bold().into(), DATABASE.tag_line.into()])
            .alignment(Alignment::Center)
            .render(content_area, buf);
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
            } else {
                self.main_menu_state.select(0);
            }
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let full = Layout::horizontal(Constraint::from_percentages([15, 85]));
        let [menu_area, display_area] = full.areas(area);

        self.render_menu(menu_area, buf);
        self.render_page(display_area, buf);
    }
}

fn main() -> color_eyre::Result<()> {
    // setup terminal
    init_error_hooks()?;
    let terminal = init_terminal()?;

    // create app and run it
    App::new().run(terminal)?;

    restore_terminal()?;

    Ok(())
}

fn init_error_hooks() -> color_eyre::Result<()> {
    let (panic, error) = HookBuilder::default().into_hooks();
    let panic = panic.into_panic_hook();
    let error = error.into_eyre_hook();
    color_eyre::eyre::set_hook(Box::new(move |e| {
        let _ = restore_terminal();
        error(e)
    }))?;
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        panic(info);
    }));
    Ok(())
}

fn init_terminal() -> color_eyre::Result<Terminal<impl Backend>> {
    crossterm::terminal::enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal() -> color_eyre::Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    std::io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
