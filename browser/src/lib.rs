use ratatui::Terminal as RatatTerm;
use ratatui_wrapper::Terminal;
use resume_tui::{App, Event, DATABASE};
use wasm_bindgen::prelude::*;

mod ratatui_wrapper;

#[wasm_bindgen]
pub fn get_resume_file_name() -> JsValue {
    JsValue::from_str(&format!("{} Resume.pdf", DATABASE.name))
}

struct Log;

impl log::Log for Log {
    fn enabled(&self, _metadata: &log::Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        let message = format!(
            "{:?} {}: {}",
            record.level(),
            record.target(),
            record.args()
        );
        web_sys::console::log_1(&JsValue::from(&message));
    }

    fn flush(&self) {}
}

#[wasm_bindgen]
pub struct BroswerApp {
    term: RatatTerm<Terminal>,
    app: App<'static>,
}

#[wasm_bindgen]
impl BroswerApp {
    pub fn tick(&mut self, event: u8) -> Result<(), JsValue> {
        web_sys::console::log_1(&JsValue::from_str(&format!("handle_event: {event}")));
        let ev = match event {
            1 => Event::Up,
            2 => Event::Down,
            3 => Event::Left,
            4 => Event::Right,
            5 => Event::Help,
            _ => {
                return Ok(self
                    .app
                    .tick(&mut self.term)
                    .map_err(|e| JsValue::from_str(&format!("RenderError: {e}")))?);
            }
        };
        self.app
            .event(ev)
            .map_err(|e| JsValue::from_str(&format!("EventError: {e}")))?;
        self.app
            .tick(&mut self.term)
            .map_err(|e| JsValue::from_str(&format!("RenderError: {e}")))?;
        Ok(())
    }
}

#[wasm_bindgen]
pub fn init_app() -> BroswerApp {
    console_error_panic_hook::set_once();
    log::set_boxed_logger(Box::new(Log)).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
    BroswerApp {
        term: ratatui::Terminal::new(ratatui_wrapper::Terminal).unwrap(),
        app: resume_tui::App::new(),
    }
}
