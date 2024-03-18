use std::sync::OnceLock;

use ratatui::Terminal as RatatTerm;
use ratatui_wrapper::Terminal;
use resume_tui::{App, Event};
use wasm_bindgen::prelude::*;

mod ratatui_wrapper;

static mut TERMINAL: OnceLock<RatatTerm<Terminal>> = OnceLock::new();
static mut APP: OnceLock<App> = OnceLock::new();

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    log::set_boxed_logger(Box::new(Log)).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
    let terminal = ratatui::Terminal::new(ratatui_wrapper::Terminal).unwrap();
    unsafe {
        TERMINAL
            .set(terminal)
            .unwrap_or_else(|_| panic!("couldn't set terminal"));
        APP.set(resume_tui::App::new()).unwrap();
    }
    Ok(())
}

#[wasm_bindgen]
pub fn event(event: u8) -> Result<(), JsValue> {
    web_sys::console::log_1(&JsValue::from_str(&format!("event {event}")));
    let Some(app) = (unsafe { APP.get_mut() }) else {
        web_sys::console::log_1(&JsValue::from_str("no APP!"));
        return Ok(());
    };
    let Some(t) = (unsafe { TERMINAL.get_mut() }) else {
        web_sys::console::log_1(&JsValue::from_str("no TERMINAL!"));
        return Ok(());
    };
    handle_event(event as _, app)?;
    web_sys::console::log_1(&JsValue::from_str("ticking!"));
    app.tick(t)
        .map_err(|e| JsValue::from_str(&format!("TickError: {e}")))
}

fn handle_event(event: u8, app: &mut App) -> Result<(), JsValue> {
    web_sys::console::log_1(&JsValue::from_str(&format!("handle_event: {event}")));
    let ev = match event {
        1 => Event::Up,
        2 => Event::Down,
        3 => Event::Left,
        4 => Event::Right,
        _ => {
            return Ok(());
        }
    };
    app.event(ev)
        .map_err(|e| JsValue::from_str(&format!("EventError: {e}")))?;
    Ok(())
}

struct Log;

impl log::Log for Log {
    fn enabled(&self, _metadata: &log::Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        let message = format!("{:?} {}: {}", record.level(), record.target(), record.args());
        web_sys::console::log_1(&JsValue::from(&message));
    }

    fn flush(&self) {}
}
