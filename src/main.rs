use color_eyre::config::HookBuilder;
use crossterm::{
    event::{self, Event as TermEvent, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use resume_tui::{App, Error, Event};

fn main() -> color_eyre::Result<()> {
    #[cfg(feature = "logging")]
    env_logger::init();
    // setup terminal
    init_error_hooks()?;
    let mut terminal = init_terminal()?;
    let mut app = App::new();
    loop {
        app.tick(&mut terminal)?;
        if let TermEvent::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                let ev = match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => Event::Quit,
                    KeyCode::Char('h') | KeyCode::Left => Event::Left,
                    KeyCode::Char('j') | KeyCode::Down => Event::Down,
                    KeyCode::Char('k') | KeyCode::Up => Event::Up,
                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => Event::Right,
                    _ => continue,
                };
                let res = app.event(ev);
                if matches!(res, Err(Error::Exit)) {
                    break;
                }
                res?;
            }
        }
    }
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
